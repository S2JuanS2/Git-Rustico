use std::fmt;
use std::io::Read;
use std::net::TcpStream;
use std::path::Path;

use crate::commands::branch::get_parent_hashes;
use crate::commands::cat_file::git_cat_file;
use crate::consts::{END_OF_STRING, VERSION_DEFAULT, CAPABILITIES_FETCH, PKT_NAK, PARENT_INITIAL};
use crate::git_server::GitServer;
use crate::git_transport::negotiation::{receive_request, receive_reference_update_request};
use crate::git_transport::references_update::send_decompressed_package_status;
use crate::util::connections::{send_message, receive_packfile};
use crate::util::errors::UtilError;
use crate::util::files::{open_file, read_file_string};
use crate::util::objects::{ObjectType, ObjectEntry};
use crate::util::packfile::send_packfile;
use crate::util::pkt_line::{add_length_prefix, read_line_from_bytes, read_pkt_line};
use crate::util::validation::join_paths_correctly;

use super::negotiation::{
    receive_done, send_acknowledge_last_reference, sent_references_valid_client,
};
use super::references::{get_objects, get_objects_fetch_with_hash_valid};
use super::references_update::{ReferencesUpdate, send_decompression_failure_status};
use super::request_command::RequestCommand;

/// # `GitRequest`
///
/// Estructura que representa una solicitud de Git, que contiene un comando, una ruta y parámetros adicionales.
///
/// `GitRequest` encapsula los componentes de una solicitud Git, incluyendo el comando de la solicitud, la ruta del repositorio
/// y los parámetros adicionales.
///
/// ## Miembros
///
/// - `request_command`: Comando de la solicitud Git, representado por un tipo `RequestCommand`.
///
/// - `pathname`: Ruta del repositorio solicitado en la solicitud Git.
///
/// - `extra_parameters`: Parámetros adicionales proporcionados en la solicitud Git.
///
#[derive(Debug, PartialEq, Eq)]
pub struct GitRequest {
    pub request_command: RequestCommand,
    pub pathname: String,
    pub extra_parameters: Vec<String>,
}

impl fmt::Display for GitRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Request Command: {}\nPathname: {}\nExtra Parameters: {:?}",
            self.request_command, self.pathname, self.extra_parameters
        )
    }
}

impl GitRequest {
    /// Lee y procesa una solicitud de Git a partir de datos leídos de un flujo de lectura.
    /// Se utiliza para leer y procesar la solicitud de un cliente Git desde un flujo de lectura.
    /// Se espera que la solicitud se reciba como un paquete de líneas de Git y, por lo tanto,
    /// se utiliza la función `read_pkt_line` para extraer los datos.
    ///
    /// # Argumentos
    ///
    /// * `listener` - Un flujo de lectura que se utiliza para obtener datos de una solicitud Git.
    ///
    /// # Retorno
    ///
    /// Devuelve un `Result` que contiene un `GitRequest` si la solicitud se procesa correctamente,
    /// o bien un error de tipo `UtilError` si la solicitud es inválida o no puede procesarse.
    ///
    pub fn read_git_request(reader: &mut dyn Read) -> Result<GitRequest, UtilError> {
        let data = read_pkt_line(reader)?;
        if data.is_empty() {
            return Err(UtilError::InvalidRequestFlush);
        }
        process_request_data(&data)
    }

    /// Crea una solicitud Git a partir de datos en bytes leídos, los bytes leidos
    /// deben tener formato pkt.
    /// Se utiliza para convertir datos de una solicitud en bytes a una estructura `GitRequest`.
    /// Está destinada a manejar los datos de solicitud ya formateados en bytes.
    ///
    /// # Argumentos
    ///
    /// * `bytes` - Datos de la solicitud Git formateados como bytes.
    ///
    /// # Retorno
    ///
    /// Devuelve un `Result` que contiene un `GitRequest` si la solicitud se procesa correctamente,
    /// o bien un error de tipo `UtilError` si la solicitud es inválida o no puede procesarse.
    ///
    pub fn create_from_bytes(bytes: &[u8]) -> Result<GitRequest, UtilError> {
        let data = read_line_from_bytes(bytes)?;
        if data.is_empty() {
            return Err(UtilError::InvalidRequestFlush);
        }
        process_request_data(data)
    }

    /// Crea una solicitud Git a partir de información detallada proporcionada.
    /// Se utiliza para construir una solicitud Git con información específica como el comando,
    /// la dirección del repositorio, la IP y el puerto del host.
    ///
    /// # Argumentos
    ///
    /// * `command` - Comando de solicitud de Git (RequestCommand).
    /// * `repo` - Nombre del repositorio.
    /// * `ip` - Dirección IP del host.
    /// * `port` - Puerto del host.
    ///
    /// # Retorno
    ///
    /// Devuelve una cadena de texto que representa la solicitud Git con la información proporcionada.
    ///
    pub fn create_from_command(
        command: RequestCommand,
        repo: String,
        ip: String,
        port: String,
    ) -> String {
        let mut len: usize = 0;

        let command = format!("{} ", command.to_string());
        len += command.len();

        let project = format!("/{}{}", repo, END_OF_STRING);
        len += project.len(); // El len cuenta el END_OF_STRING

        let host = format!("host={}:{}{}", ip, port, END_OF_STRING);
        len += host.len(); // El len cuenta el END_OF_STRING

        let message = format!("{}{}{}", command, project, host);
        add_length_prefix(&message, len)
    }

    /// Crea una solicitud Git con los datos especificados.
    ///
    /// Esta función genera una solicitud Git formateada como una cadena, basada en el comando de solicitud,
    /// el repositorio, la dirección IP y el puerto proporcionados como argumentos. La solicitud incluye
    /// información sobre el comando, el proyecto (repositorio) y el host (IP y puerto).
    ///
    /// ## Argumentos
    ///
    /// - `command`: El comando de solicitud Git (`RequestCommand`) que se utilizará.
    /// - `repo`: El nombre del repositorio en el que se realizará la solicitud.
    /// - `ip`: La dirección IP del host al que se enviará la solicitud.
    /// - `port`: El puerto en el que se realizará la conexión con el host.
    ///
    /// ## Retorno
    ///
    /// Una line pkt que representa la solicitud Git formateada.
    pub fn generate_request_string(
        command: RequestCommand,
        repo: &str,
        ip: &str,
        port: &str,
    ) -> String {
        let mut len: usize = 0;

        let command = format!("{} ", command.to_string());
        len += command.len();

        let project = format!("/{}{}", repo, END_OF_STRING);
        len += project.len(); // El len cuenta el END_OF_STRING

        let host = format!("host={}:{}{}", ip, port, END_OF_STRING);
        len += host.len(); // El len cuenta el END_OF_STRING

        let message = format!("{}{}{}", command, project, host);
        add_length_prefix(&message, len)
    }

    pub fn execute(&self, stream: &mut TcpStream, root: &str) -> Result<String, UtilError> {
        match self.request_command {
            RequestCommand::UploadPack => {
                let path_repo = get_path_repository(root, &self.pathname)?;
                handle_upload_pack(stream, &path_repo)
            }
            RequestCommand::ReceivePack => {
                let path_repo = get_path_repository(root, &self.pathname)?;
                handle_receive_pack(stream, &path_repo)?;
                println!("ReceivePack");
                println!("Funcion aun no implementada");
                Ok("".to_string())
            }
            RequestCommand::UploadArchive => {
                println!("Funcion aun no implementada");
                println!("UploadArchive");
                Ok("".to_string())
            }
        }
    }
}

fn handle_upload_pack(stream: &mut TcpStream, path_repo: &str) -> Result<String, UtilError> {
    let capabilities: Vec<String> = CAPABILITIES_FETCH.iter().map(|&s| s.to_string()).collect();
    let mut server = GitServer::create_from_path(path_repo, VERSION_DEFAULT, &capabilities)?;
    // println!("Server: {:?}", server);
    server.send_references(stream)?;
    // println!("Envie las referencias");
    let (capabilities, wanted_objects, had_objects) = receive_request(stream)?;
    if capabilities.is_empty() && wanted_objects.is_empty() && had_objects.is_empty() {
        return Ok("No solicito referencias".to_string());
    }

    if !had_objects.is_empty() {
        // Si el cliente cuenta con objetos ya en su repo, esta haciendo un FETCH

        server.update_data(capabilities, wanted_objects);
        let local_hashes = search_available_references(path_repo, &had_objects);
        
        // Las referencias al dia las filtro
        server.filter_available_references(&local_hashes);
        sent_references_valid_client(stream, &local_hashes)?;
        // Confirmo las referencias del usuario que el servidor tiene disponibles
        // Actualizo las referencias disponibles del servidor
        // server.update_local_references(&local_references);

        // Las confirmaciones terminan con recibiendo un done
        receive_done(stream, UtilError::ReceiveDoneConfRefs)?;

        // Envio el ultimo ACK
        send_acknowledge_last_reference(stream, &local_hashes)?;
        
        let objects = get_objects_fetch(&mut server, local_hashes)?;
        send_packfile(stream, &server, objects, true)?;

        return Ok("Fetch exitoso".to_string());
    }
    // Si el cliente solicita todo, esta haciendo un CLONE
    server.update_data(capabilities, wanted_objects);
    let objects = match get_objects(path_repo, &server.available_references[1..]) {
        Ok(objects) => objects,
        Err(_) => return Err(UtilError::GetObjectsPackfile),
    };
    send_message(stream, PKT_NAK, UtilError::SendNAKPackfile)?;
    send_packfile(stream, &server, objects, true)?; // Debo modificarlo, el NAK no debe estar dentro
    Ok("Clone exitoso".to_string())
}


// [TODO #4]
// Dado las referencias(local_hash: Vector de hashes) que el cliente supuestamente tiene
// Se deben filtrar los hash que tiene el servidor
// Me debes devolver un Vec<String> con los hash que tenemos en comun
// Acordate que el repo esta en path_repo
pub fn search_available_references(
    path_repo: &str,
    local_hash: &Vec<String>,
) -> Vec<String> {
    let mut confirmed_commits: Vec<String> = Vec::new();
    let commits_in_repo = match get_commits(path_repo) {
        Ok(commits) => commits,
        Err(_) => return confirmed_commits,
    };
    for hash in local_hash {
        // Ejemplo de como seria
        // if reference.la_tenemos() {
        //     available_references.push(reference);
        // }
        println!("AQUI IRIA TU TODO");

        if commits_in_repo.contains(hash) {
            confirmed_commits.push(hash.to_string());
        }
    }
    confirmed_commits
}

fn get_commits(path_repo: &str) -> Result<Vec<String>, UtilError> {
    let mut commits: Vec<String> = Vec::new();
    let branches_path = join_paths_correctly(path_repo, ".git/refs/heads");
    let branches = match std::fs::read_dir(branches_path) {
        Ok(branches) => branches,
        Err(_) => return Err(UtilError::ReadDirError),
    };
    for branch in branches {
        let branch = match branch {
            Ok(branch) => branch,
            Err(_) => return Err(UtilError::ReadDirError),
        };
        let branch_name = branch.file_name();
        if let Some(branch_name) = branch_name.to_str() {
            let branch_path = join_paths_correctly(path_repo, &format!(".git/refs/heads/{}", branch_name));
            let branch_file = open_file(&branch_path)?;
            let branch_content = read_file_string(branch_file)?;
            recover_commits(path_repo, &branch_content, &mut commits)?;
        }
    }

    Ok(commits)
}

fn recover_commits(path_repo: &str, branch_content: &str, mut commits: &mut Vec<String>) -> Result<(), UtilError> {
    commits.push(branch_content.to_string());
    let commit_content = git_cat_file(path_repo, branch_content, "-p")?;
    let parent_commit = get_parent_hashes(commit_content);
    if parent_commit == PARENT_INITIAL {
        return Ok(());
    }
    recover_commits(path_repo, &parent_commit, &mut commits)?;
    Ok(())
}

/// Procesa los datos de una solicitud Git y los convierte en una estructura `GitRequest`.
/// Esta función toma los datos de la solicitud Git y los divide en comandos y argumentos.
///
/// # Argumentos
///
/// * `data` - Los datos de la solicitud Git que se van a procesar.
///
/// # Retorno
///
/// Devuelve un `Result` que contiene un `GitRequest` si la solicitud se procesa correctamente,
/// o bien un error de tipo `UtilError` si la solicitud es inválida o no puede procesarse.
///
fn process_request_data(data: &[u8]) -> Result<GitRequest, UtilError> {
    let (first_part, second_part) = if let Some(idx) = data.iter().position(|&byte| byte == b' ') {
        let (first, second) = data.split_at(idx);
        (first, &second[1..])
    } else {
        return Err(UtilError::InvalidRequestCommand);
    };

    let request_command = RequestCommand::from_string(first_part)?;

    get_components_request(second_part).map(|(pathname, extra_parameters)| GitRequest {
        request_command,
        pathname: String::from_utf8_lossy(pathname).trim().to_string(),
        extra_parameters,
    })
}

/// Obtiene los componentes de una solicitud Git y los retorna como tupla.
/// Toma los bytes de una solicitud Git y los separa en sus diferentes componentes,
/// devolviendo una tupla que contiene el pathname y los parámetros adicionales.
///
/// # Argumentos
///
/// * `bytes` - Bytes de la solicitud Git que se van a dividir en componentes.
///
/// # Retorno
///
/// Devuelve un `Result` que contiene una tupla con los componentes si la solicitud se procesa correctamente,
/// o bien un error de tipo `UtilError` si la solicitud es inválida o no puede procesarse.
///
fn get_components_request(bytes: &[u8]) -> Result<(&[u8], Vec<String>), UtilError> {
    let mut components = bytes.split(|&byte| byte == 0);

    let pathname = match components.next() {
        Some(path) => path,
        None => return Err(UtilError::InvalidRequestCommandMissingPathname),
    };

    let extra_parameters = components.collect::<Vec<_>>();
    Ok((
        pathname,
        extra_parameters
            .iter()
            .map(|p| String::from_utf8_lossy(p).trim().to_string())
            .collect(),
    ))
}

/// Obtiene la ruta del repositorio dado un directorio raíz y un nombre de ruta.
///
/// # Argumentos
///
/// * `root` - Ruta del directorio raíz.
/// * `pathname` - Nombre de la ruta del repositorio.
///
/// # Retorna
///
/// Devuelve un resultado que contiene la ruta del repositorio si la operación es exitosa.
/// En caso de error, retorna un error de tipo UtilError indicando la no existencia del repositorio.
///
fn get_path_repository(root: &str, pathname: &str) -> Result<String, UtilError> {
    let path_repo = join_paths_correctly(root, pathname);
    let path = Path::new(&path_repo);
    println!("{:?}", path);
    if !(path.exists() && path.is_dir()) {
        return Err(UtilError::RepoNotFoundError(pathname.to_string()));
    }
    // Valido si es un repo git
    let path_git = join_paths_correctly(&path_repo, ".git");
    let path = Path::new(&path_git);
    if !(path.exists() && path.is_dir()) {
        return Err(UtilError::RepoNotFoundError(pathname.to_string()));
    }
    Ok(path_repo)
}


// [TODO #7]
// Pero la diferencia de esta funcion, es que no tengo que enviar todos los objs.
// Las referencias que tengo que enviar son las que el cliente no tiene
// Estan estan en &git_server.available_references
// Y los hash que el cliente tiene y nosotros validamos que ya teniamos estan en _confirmed_hashes
// Te elimino el HEAD en las referencias por posibles bugs
pub fn get_objects_fetch(git_server: &mut GitServer, confirmed_hashes: Vec<String>) -> Result<Vec<(ObjectType, Vec<u8>)>,UtilError>
{
    git_server.delete_head_in_available_references();
    let references = &git_server.available_references;
    let objects = get_objects_fetch_with_hash_valid(
        &git_server.src_repo,
        references.to_vec(),
        &confirmed_hashes
        )?;

    Ok(objects)
}


pub fn handle_receive_pack(stream: &mut TcpStream, path_repo: &str) -> Result<(), UtilError>{
    let mut server = GitServer::create_from_path(path_repo, VERSION_DEFAULT, &Vec::new())?;
    server.send_references(stream)?;

    let requests = receive_reference_update_request(stream, &mut server)?;
    let objects = receive_packfile(stream)?;

    match process_request_update(requests, objects)
    {
        Ok(status) => send_decompressed_package_status(stream, &status),
        Err(_) => send_decompression_failure_status(stream),
    }
}


// [TODO #8]
// Esta funcion es la que se encarga de procesar las actualizaciones de las referencias
// Y de actualizar el repo
// Recibe un vector de ReferencesUpdate y un vector de (ObjectEntry, Vec<u8>)
// El vector de ReferencesUpdate son las referencias que el cliente quiere actualizar
// Atributos de ReferencesUpdate:
// - old: String -> Hash del objeto viejo
// - new: String -> Hash del objeto nuevo
// - path_refs: String -> Ruta de la referencia -> Ejemplo: refs/heads/master
// ReferencesUpdate tambien se usara para saber lo que el cliente quiere hacer:
//   create branch     =  old-id=zero-id  new-id 
//   delete branch     =  old-id          new-id=zero-id
//   update branch     =  old-id          new-id 
// Se puede devolvera un vector del tipo Vec<(String, bool)>
// Donde el String es la referencia y el bool es si fue exitosa o no
// ESto se usara para decirle al cliente si la actualizacion fue exitosa o no
// EL falso es solo si no se puede actualizar
// SI el paquete esta corrupto se debe enviar un error
pub fn process_request_update(
    _requests: Vec<ReferencesUpdate>,
    _objects: Vec<(ObjectEntry, Vec<u8>)>,
) -> Result<Vec<(String, bool)>, UtilError> {
    return Ok(Vec::new());
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_request_string_upload_pack() {
        let message = GitRequest::generate_request_string(
            RequestCommand::UploadPack,
            "project.git",
            "myserver.com",
            "9418",
        );
        assert_eq!(
            message,
            "0038git-upload-pack /project.git\0host=myserver.com:9418\0"
        );
    }

    #[test]
    fn test_generate_request_string_receive_pack() {
        let message = GitRequest::generate_request_string(
            RequestCommand::ReceivePack,
            "project.git",
            "127.0.0.2",
            "12030",
        );
        assert_eq!(
            message,
            "0037git-receive-pack /project.git\0host=127.0.0.2:12030\0"
        );
    }

    #[test]
    fn test_generate_request_string_upload_archive() {
        let message = GitRequest::generate_request_string(
            RequestCommand::UploadArchive,
            "project.git",
            "250.250.250.250",
            "8080",
        );
        assert_eq!(
            message,
            "003egit-upload-archive /project.git\0host=250.250.250.250:8080\0"
        );
    }

    #[test]
    fn test_git_request_new_with_valid_format() {
        // Datos de entrada válidos con un espacio
        let input = b"003agit-upload-pack /schacon/gitbook.git\0host=example.com\0";
        let result = GitRequest::create_from_bytes(input);
        assert!(result.is_ok()); // Comprobar que el resultado es Ok
    }

    #[test]
    fn test_git_request_new_with_invalid_format() {
        // Datos de entrada sin espacio entre command y pathname
        let input_no_space = b"003agit-upload-pack/schacon/gitbook.git\0host=example.com\0";
        let result_no_space = GitRequest::create_from_bytes(input_no_space);
        assert!(result_no_space.is_err()); // Comprobar que el resultado es un error
    }

    #[test]
    fn test_git_request_new_with_invalid_lenght() {
        let input_no_space = b"013agit-upload-pack/schacon/gitbook.git\0host=example.com\0";
        let result_no_space = GitRequest::create_from_bytes(input_no_space);
        assert!(result_no_space.is_err()); // Comprobar que el resultado es un error
    }
    #[test]
    fn test_git_request_new_with_valid_data() -> Result<(), UtilError> {
        // Datos de entrada con un comando no válido
        let input_invalid_command = b"003agit-upload-pack /schacon/gitbook.git\0host=example.com\0";
        let request = GitRequest::create_from_bytes(input_invalid_command)?;
        assert!(request.request_command == RequestCommand::UploadPack);
        assert!(request.pathname == "/schacon/gitbook.git");
        assert!(vec![String::from("host=example.com")].eq(&request.extra_parameters));
        Ok(())
    }
}
