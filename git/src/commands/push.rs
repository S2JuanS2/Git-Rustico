use super::cat_file::git_cat_file;
use super::checkout::extract_parent_hash;
use super::errors::CommandsError;
use crate::commands::config::GitConfig;
use crate::consts::ZERO_ID;
use crate::git_transport::git_request::GitRequest;
use crate::git_transport::references::{Reference, reference_discovery, get_objects_from_hash_to_hash};
use crate::git_transport::request_command::RequestCommand;
use crate::models::client::Client;
use crate::util::connections::{start_client, send_message, send_flush};
use crate::util::errors::UtilError;
use crate::util::pkt_line;
use crate::util::packfile::send_packfile;
use std::net::TcpStream;


pub struct PushBranch{
    pub path_local: String,
    pub remote_name: String,
    pub url_remote: String,
    pub git_config: GitConfig,
    pub branch: Reference,
    pub status: Vec<String>
}

impl PushBranch
{
    fn new(path_local: String, name_branch: &str) -> Result<Self, CommandsError>
    {
        // Obtengo el repositorio remoto
        let git_config = GitConfig::new_from_file(&path_local)?;
        let branch = Reference::create_from_name_branch(&path_local, name_branch)?;
        let remote_name = git_config.get_remote_by_branch_name(branch.get_name())?;
        let url_remote = git_config.get_remote_url_by_name(&remote_name)?;
        let status = Vec::new();
        let mut push = PushBranch{path_local, remote_name, url_remote, git_config, branch, status};
        push.init_status();
        Ok(push)
    }

    fn init_status(&mut self)
    {
        self.status.push(format!("Pushing to {}...", &self.branch.get_name()));
        self.status.push(format!("\tRepositorio local {}", &self.path_local));
        self.status.push(format!("\tRepositorio remoto {}", &self.remote_name));
        self.status.push(format!("\tUrl del remoto: {}", &self.url_remote));
    }

    fn add_status(&mut self, status: &str)
    {
        self.status.push(status.to_string());
    }

    fn _get_status(&self) -> String
    {
        self.status.join("\n")
    }

    fn get_hash(&self) -> String
    {
        self.branch.get_hash().to_string()
    }
    fn _get_path_local(&self) -> String
    {
        self.path_local.to_string()
    }
}
/// Comandos que aceptare:
/// git push -> push de la rama actual
/// git push remote branch -> si la branch actual no tiene le agregamos el remote
/// Maneja el comando "push" en el servidor Git.
/// 
/// # Arguments
///
/// * `args` - Argumentos proporcionados al comando. Se espera que esté vacío ya que "push" no requiere argumentos.
/// * `client` - Objeto `Client` que contiene la información del cliente, como la dirección, el puerto y la ruta del directorio.
///
/// # Returns
///
/// Retorna un resultado que indica si la operación "push" fue exitosa o si hubo errores durante la ejecución.
///
/// # Errors
///
/// Retorna un error si la cantidad de argumentos no es la esperada o si hay problemas al iniciar la conexión con el cliente o ejecutar el comando "git push".
///
pub fn handle_push(args: Vec<&str>, client: Client) -> Result<String, CommandsError> {
    if !(args.is_empty() || args.len() == 2) {
        return Err(CommandsError::InvalidArgumentCountPush);
    }
    
    let path_local = client.get_directory_path();
    let mut socket = start_client(client.get_address())?;
    
    if args.is_empty() {
        let name_branch = get_name_current_branch(path_local)?;
        return git_push_branch (
            &mut socket,
            client.get_ip(),
            client.get_port(),
            &mut PushBranch::new(path_local.to_string(), &name_branch)?,
        );
    }
    if args[1] != "all" {
        return Err(CommandsError::InvalidArgumentPush);
    }
    git_push_all(
        &mut socket,
        client.get_ip(),
        client.get_port(),
        path_local,
    )
}

/// actualiza el repositorio remoto con los cambios del repositorio local
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'socket': socket del cliente
/// 'ip': ip del cliente
/// 'port': puerto del cliente
/// 'remote_name': nombre del repositorio remoto
/// 'branch_name': nombre de la rama a mergear
// Quiero actualizar mi branch actual con los cambios del repositorio remoto
pub fn git_push_branch(
    socket: &mut TcpStream,
    ip: &str,
    port: &str,
    push: &mut PushBranch
) -> Result<String, CommandsError> {
    // Prepara la solicitud "git-upload-pack" para el servidor
    let message =
        GitRequest::generate_request_string(RequestCommand::ReceivePack, &push.url_remote, ip, port);
    
    let server = reference_discovery(socket, message, &push.url_remote, &Vec::new())?;
    let prev_hash = match server.get_remote_reference_hash(push.branch.get_ref_path())
    {
        Some(hash) => hash, // Actualizo en el remoto
        None => ZERO_ID.to_string(), // Creo en el remoto
    };

    println!("Prev hash: {}", prev_hash);
    let current_hash = push.get_hash(); // Commit local
    println!("Current hash: {}", current_hash);
    
    // if !is_necessary_to_update(push, &current_hash, &prev_hash)?
    // {
    //     return Ok(push.get_status());
    // }
    println!("Necesito actualizar");
    // AViso que actualizare mi branch
    reference_update(socket, &prev_hash, &current_hash, push.branch.get_ref_path())?;
    println!("Se actualizo la referencia");
    
    let objects = get_objects_from_hash_to_hash(&push.path_local, &prev_hash, &current_hash)?;
    send_packfile(socket, &server, objects, true)?;
    push.add_status("Se envio el packfile ...");

    // No se que me enviara el servidor
    // Por eso leere todo para examinar la respuesta de daemon
    let lines = pkt_line::read(socket)?;
    for line in lines
    {
        println!("Line in string: {}", String::from_utf8(line).unwrap());
    }

    Ok("Hola, soy baby push!".to_string())
}


fn git_push_all(
    _socket: &mut TcpStream,
    _ip: &str,
    _port: &str,
    _path_local: &str,
) -> Result<String, CommandsError> {
    Ok("Hola, soy baby push all!".to_string())
}

/// Obtiene el nombre de la rama actual en un repositorio Git local.
///
/// # Argumentos
///
/// * `path_repo`: Ruta al directorio del repositorio Git.
///
/// # Devuelve
///
/// Un `Result<String, UtilError>` que contiene el nombre de la rama actual si la operación fue exitosa.
/// En caso de error, se devuelve un detalle específico en el tipo `UtilError`.
///
/// # Ejemplo
///
fn get_name_current_branch(path_repo: &str) -> Result<String, UtilError>
{
    let path: String = format!("{}/.git/HEAD", path_repo);
    let head = match std::fs::read_to_string(path) {
        Ok(head) => head,
        Err(_) => return Err(UtilError::CurrentBranchNotFound),
    };
    let ref_path = match head.split(':').last()
    {
        Some(ref_path) => ref_path,
        None => return Err(UtilError::CurrentBranchNotFound)
    };
    match ref_path.trim().split('/').last()
    {
        Some(name) => Ok(name.to_string()),
        None => Err(UtilError::CurrentBranchNotFound)
    }
}


/// Determina si es necesario realizar una operación de actualización (push) en el servidor remoto.
///
/// # Argumentos
///
/// * `push`: Referencia mutable a un objeto `PushBranch` utilizado para almacenar mensajes de estado.
/// * `hash_current`: Hash del commit actual en la rama local.
/// * `hash_prev`: Hash del commit previo en la rama remota.
///
fn _is_necessary_to_update(push: &mut PushBranch, hash_current: &str, hash_prev: &str) -> Result<bool, CommandsError>
{
    if hash_current == hash_prev
    {
        push.add_status("No hay cambios que subir");
        return Ok(false)
    }
    if is_ancestor(&push._get_path_local(), hash_current, hash_prev)?
    {
        push.add_status("No hay cambios que subir");
        push.add_status("Esta atrasado ...");
        push.add_status("Haga pull :)");
        return Ok(false)
    };
    Ok(true)
}


pub fn is_ancestor(directory: &str, hash_current: &str, hash_prev: &str) -> Result<bool, CommandsError>
{
    if hash_prev == ZERO_ID
    {
        return Ok(false);
    }

    if hash_current == hash_prev
    {
        return Ok(false)
    }

    let commit = git_cat_file(directory, hash_current, "-p")?;
    if let Some(parent_hash) = extract_parent_hash(&commit){
        if hash_prev == parent_hash || is_ancestor(directory, parent_hash, hash_prev)?{
            return Ok(true)
        }
    };

    // [TODO #5]
    // Si el commit local no es ancestro del commit remoto, no se puede hacer push
    // Se debe hacer pull
    // Implementar la logica de ancestro
    Ok(false)
}

/// Actualiza una referencia en el servidor Git con los hashes de commits proporcionados.
///
/// # Argumentos
///
/// * `socket`: Referencia mutable a un flujo TCP utilizado para la comunicación con el servidor.
/// * `hash_prev`: Hash del commit previo asociado con la referencia.
/// * `hash_update`: Hash del commit actualizado para la referencia.
/// * `path_ref`: Ruta de la referencia que se actualizará en el servidor Git.
///
/// # Devuelve
///
/// Un `Result<(), CommandsError>` que indica si la operación de actualización de referencia fue exitosa o si ocurrió un error.
/// En caso de error, se proporciona un detalle específico en el tipo `CommandsError`.
///
fn reference_update(socket: &mut TcpStream, hash_prev: &str, hash_update: &str, path_ref: &str) -> Result<(), CommandsError>
{   
    let message = format!("{} {} {}\n", hash_prev, hash_update, path_ref);
    let message = pkt_line::add_length_prefix(&message, message.len());
    send_message(socket, &message, UtilError::SendMessageReferenceUpdate)?;
    send_flush(socket, UtilError::SendMessageReferenceUpdate)?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::commands::{add::git_add, init::git_init, commit::*};
    use crate::util::files::{read_file_string, open_file};

    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn commit_test() {
        let directory = "./test_commit_repo";
        git_init(directory).expect("Falló en el comando init");

        let file_path = format!("{}/{}", directory, "holamundo.txt");
        let mut file = fs::File::create(&file_path).expect("Falló al crear el archivo");
        file.write_all(b"Hola Mundo")
            .expect("Error al escribir en el archivo");

        let file_path = format!("{}/{}", directory, "chaumundo.txt");
        let mut file = fs::File::create(&file_path).expect("Falló al crear el archivo");
        file.write_all(b"Chau Mundo")
            .expect("Error al escribir en el archivo");

        let file_path = format!("{}/{}", directory, "himundo.txt");
        let mut file = fs::File::create(&file_path).expect("Falló al crear el archivo");
        file.write_all(b"Hi Mundo")
            .expect("Error al escribir en el archivo");

        let test_commit = Commit::new(
            "prueba".to_string(),
            "Juan".to_string(),
            "jdr@fi.uba.ar".to_string(),
            "Juan".to_string(),
            "jdr@fi.uba.ar".to_string(),
        );
        let branch = format!("{}/.git/refs/heads/master", directory);

        git_add(directory, "holamundo.txt").expect("Fallo en el comando add");

        git_commit(directory, test_commit.clone()).expect("Error commit");

        let file_branch = open_file(&branch).expect("Error open file");
        let prev_hash = read_file_string(file_branch).expect("Error read file");

        git_add(directory, "chaumundo.txt").expect("Fallo en el comando add");

        git_commit(directory, test_commit.clone()).expect("Error Commit");

        git_add(directory, "himundo.txt").expect("Fallo en el comando add");

        git_commit(directory, test_commit).expect("Error Commit");

        let file_branch = open_file(&branch).expect("Error open file");
        let hash_current = read_file_string(file_branch).expect("Error read file");

        let result = is_ancestor(directory, &hash_current, &prev_hash).expect("Error ancestor");

        fs::remove_dir_all(directory).expect("Falló al remover los directorios");

        assert_eq!(result, true)
    }
}