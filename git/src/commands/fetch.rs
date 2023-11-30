use crate::commands::config::GitConfig;
use crate::commands::fetch_head::FetchHead;
use crate::consts::{DIRECTORY, FILE, GIT_DIR, CAPABILITIES_FETCH, DIR_OBJECTS};
use crate::git_server::GitServer;
use crate::git_transport::negotiation::packfile_negotiation_partial;
use crate::git_transport::references::{reference_discovery, Reference};
use crate::git_transport::request_command::RequestCommand;
use crate::models::client::Client;
use crate::util::connections::{receive_packfile, start_client, send_flush};
use crate::util::errors::UtilError;
use crate::util::files::{ensure_directory_clean, create_directory};
use crate::util::objects::{
    builder_object_blob, builder_object_commit, builder_object_tree, read_blob, read_commit,
    read_tree, ObjectEntry, ObjectType,
};
use crate::git_transport::git_request::GitRequest;
use crate::util::pkt_line::read_pkt_line;
use std::net::TcpStream;
use std::path::Path;
use std::fs;

use super::errors::CommandsError;

// use super::cat_file::git_cat_file;

// const REMOTES_DIR: &str = "refs/remotes/";

/// Maneja la ejecución del comando "fetch" en el cliente Git.
///
/// # Developer
///
/// Solo se aceptaran los comandos que tengan la siguiente estructura:
///
/// * `git fetch`
///
/// # Argumentos
///
/// * `args`: Un vector que contiene los argumentos pasados al comando "fetch". En este caso, se espera que esté vacío, ya que solo se admite la forma básica `git fetch`.
///
/// * `client`: Un objeto `Client` que representa la configuración del cliente Git.
///
/// # Retorno
///
/// Devuelve un `Result` que contiene `Ok(())` en caso de éxito o un error (CommandsError) en caso de fallo.
///
/// # Errores
///
/// * Otros errores de `CommandsError`: Pueden ocurrir errores relacionados con la conexión al servidor Git, la inicialización del socket o el proceso de fetch.
///
pub fn handle_fetch(args: Vec<&str>, client: Client) -> Result<String, CommandsError> {
    println!("Entre al handle fetch");
    if args.len() >= 2 {
        return Err(CommandsError::InvalidArgumentCountFetchError);
    }
    let mut socket = start_client(client.get_address())?;
    if args.is_empty() {
        return git_fetch_all(
            &mut socket,
            client.get_ip(),
            client.get_port(),
            client.get_directory_path(),
        );
    }
    git_fetch_branch(
        &mut socket,
        client.get_ip(),
        client.get_port(),
        client.get_directory_path(),
        args[0],
    )
}

pub fn git_fetch_all(
    socket: &mut TcpStream,
    ip: &str,
    port: &str,
    repo_local: &str,
) -> Result<String, CommandsError> {
    // Obtengo el repositorio remoto
    println!("Repositorio local: {}", repo_local);
    let git_config = GitConfig::new_from_file(repo_local)?;
    let repo_remoto = git_config.get_remote_repo()?;

    println!("Fetch del repositorio remoto: {}", repo_remoto);

    // Prepara la solicitud "git-upload-pack" para el servidor
    let message =
        GitRequest::generate_request_string(RequestCommand::UploadPack, repo_remoto, ip, port);

    // Reference Discovery
    let my_capacibilities:Vec<String> = CAPABILITIES_FETCH.iter().map(|&s| s.to_string()).collect();
    let mut server = reference_discovery(socket, message, repo_remoto, &my_capacibilities)?;
    
    // Packfile Negotiation
    packfile_negotiation_partial(socket, &mut server, repo_local)?;

    // Packfile Data
    let _last_ack = read_pkt_line(socket)?; // Vlidar last ack
    let content = receive_packfile(socket)?;

    if content.is_empty()
    {
        return Ok("No hay nuevas actualizaciones. Todo está actualizado.".to_string());
    }
    
    if save_objects(content, repo_local).is_err() {
        return Err(CommandsError::RepositoryNotInitialized);
    };

    let refs = server.get_references_for_updating()?;
    save_references(&refs, repo_local)?;
    let fetch_head = FetchHead::new(&refs, repo_remoto)?;
    fetch_head.write(repo_local)?;

    Ok("El fetch se completó exitosamente. Se recuperaron nuevas actualizaciones.".to_string())
}

pub fn git_fetch_branch(
    socket: &mut TcpStream,
    ip: &str,
    port: &str,
    repo_local: &str,
    name_branch: &str,
) -> Result<String, CommandsError> {
    // Obtengo el repositorio remoto
    println!("Repositorio local: {}", repo_local);
    let git_config = GitConfig::new_from_file(repo_local)?;
    let repo_remoto = git_config.get_remote_repo()?;

    println!("Fetch del repositorio remoto: {}", repo_remoto);
    let rfs_fetch = format!("refs/heads/{}", name_branch);

    // Prepara la solicitud "git-upload-pack" para el servidor
    let message =
        GitRequest::generate_request_string(RequestCommand::UploadPack, repo_remoto, ip, port);

    // Reference Discovery
    let my_capacibilities:Vec<String> = CAPABILITIES_FETCH.iter().map(|&s| s.to_string()).collect();
    let mut server = reference_discovery(socket, message, repo_remoto, &my_capacibilities)?;
    if !server.contains_reference(&rfs_fetch)
    {
        send_flush(socket, UtilError::SendFlushCancelConnection)?;
        return Ok(format!("No existe la branch {} en el repositorio remoto.", name_branch))
    }

    // Packfile Negotiation
    // Solo solicitar una branch
    server.filter_references_for_update([rfs_fetch].to_vec())?;
    packfile_negotiation_partial(socket, &mut server, repo_local)?;

    // Packfile Data
    let _last_ack = read_pkt_line(socket)?; // Vlidar last ack
    let content = receive_packfile(socket)?;

    if content.is_empty()
    {
        return Ok("No hay nuevas actualizaciones. Todo está actualizado.".to_string());
    }

    if save_objects(content, repo_local).is_err() {
        println!("Error al guardar los objetos");
        return Err(CommandsError::RepositoryNotInitialized);
    };

    let refs = server.get_references_for_updating()?;
    save_references(&refs, repo_local)?;
    let fetch_head = FetchHead::new(&refs, repo_remoto)?;
    fetch_head.write(repo_local)?;

    Ok("Sucessfully!".to_string())
}


/// Devuelve las referencias (nombres de las branches y hashes)
///  
/// # Argumentos
///
/// * `server`: Contiene las referencias recibidas del servidor
///
/// # Errores
///
/// Devuelve un error del tipo `CommandsError` si hay problemas
///
pub fn get_branches(server: &GitServer) -> Result<Vec<(String,String)>,CommandsError> {

    let mut references: Vec<(String,String)> = vec![];

    for reference in server.get_references().iter().skip(1){
        let hash = reference.get_hash();
        let branch = reference.get_ref_path();
            if let Some(current_branch) = branch.rsplit('/').next() {
                let new_ref:(String,String) = (current_branch.to_string(),hash.to_string());
                references.push(new_ref);
            }
    }
    Ok(references)
}


/// Guarda referencias (nombres y hashes) en archivos individuales dentro del directorio de referencias
/// remotas en un repositorio Git.
///
/// Esta función toma un vector de tuplas `(String, String)` que representa pares de nombres y hashes de
/// referencias. El path del repositorio `repo_path` se utiliza para construir la ruta del directorio de
/// referencias y, a continuación, se asegura de que el directorio esté limpio. Luego, escribe cada
/// par de nombre y hash en archivos individuales dentro del directorio.
///
/// # Errores
///
/// - Si no puede asegurar que el directorio de referencias esté limpio o no puede escribir en los archivos,
///   se devuelve un error del tipo `CommandsError::RemotoNotInitialized`.
///
fn save_references(references: &Vec<Reference>, repo_path: &str) -> Result<(), CommandsError> {

    // Si no existe el directorio .git/refs/remotes lo crea
    let directory_remotes = format!("{}/.git/refs/remotes", repo_path); 
    let directory_remotes = Path::new(&directory_remotes);
    create_directory(directory_remotes)?;

    // Si no existe el directorio .git/refs/remotes/origin lo crea
    let refs_dir_path = format!("{}/.git/refs/remotes/origin", repo_path);
    ensure_directory_clean(&refs_dir_path)?;

    // Escribe los hashes en archivos individuales
    for reference in references {
        let name = reference.get_name();
        let hash = reference.get_hash();
        let file_path = format!("{}/{}", refs_dir_path, name);
        if fs::write(&file_path, hash).is_err() {
            return Err(CommandsError::RemotoNotInitialized);
        };
    }

    Ok(())
}

/// Maneja la creación y el guardado de los objetos recibidos del servidor
///
/// # Argumentos
///
/// * `repo`: Contiene la dirección del repositorio al utilizar el comando fetch.
///
/// * `content`: Contiene el contenido de los objetos a crear.
///
/// # Retorno
///
/// Devuelve un `Result` que contiene `Ok(())` en caso de éxito o un error (CommandsError) en caso de fallo.
///
fn save_objects(content: Vec<(ObjectEntry, Vec<u8>)>, git_dir: &str) -> Result<(), CommandsError> {
    // Cantidad de objetos recibidos
    let count_objects = content.len();

    let path_dir_cloned = Path::new(git_dir);
    let git_dir = format!("{}/{}", git_dir, GIT_DIR);

    let mut i = 0;
    while i < count_objects {
        if content[i].0.obj_type == ObjectType::Commit {
            handle_commit(&content, &git_dir, i)?;
            i += 1;
        } else if content[i].0.obj_type == ObjectType::Tree {
            i = handle_tree(&content, &git_dir, i, path_dir_cloned)?;
            i += 1;
        }
        println!("i: {}", i);
    }
    Ok(())
}

fn recovery_blob(
    hash: &str,
    content: &Vec<(crate::util::objects::ObjectEntry, Vec<u8>)>,
    mut i: usize,
    repo: &str,
) -> Result<usize, CommandsError> {
    if i < content.len(){
        let blob_content = read_blob(&content[i].1)?;
        let blob_content_bytes = blob_content.clone();
        let object_dir = format!("{}/{}/{}/{}", repo, DIR_OBJECTS, &hash[..2], &hash[2..]);
        if !Path::new(&object_dir).exists(){
            builder_object_blob(blob_content_bytes.into_bytes(), repo)?;
        }else{
            i -= 1;
        }
    }
    Ok(i)
}

fn recovery_tree(
    tree_content: String,
    path_dir_repo: &Path,
    content: &Vec<(crate::util::objects::ObjectEntry, Vec<u8>)>,
    mut i: usize,
    repo: &str,
) -> Result<usize, CommandsError> {
    for line in tree_content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let mode;
        let file_name;
        if parts[0] == FILE || parts[0] == DIRECTORY {
            mode = parts[0];
            file_name = parts[1];
        }else{
            file_name = parts[0];
            mode = parts[1];
        }
        let hash = parts[2];

        let path_dir_repo = path_dir_repo.join(file_name);
        if mode == FILE {
            i += 1;
            i = recovery_blob(hash, content, i, repo)?;
        } else if mode == DIRECTORY {
            i += 1;
            let tree_content = read_tree(&content[i].1)?;
            builder_object_tree(repo, &tree_content)?;
            i = recovery_tree(tree_content, &path_dir_repo, content, i, repo)?;
        }
    }
    Ok(i)
}

fn handle_commit(
    content: &[(ObjectEntry, Vec<u8>)],
    git_dir: &str,
    i: usize,
) -> Result<(), CommandsError> {
    let commit_content = read_commit(&content[i].1)?;
    builder_object_commit(&commit_content, git_dir)?;
    Ok(())
}

fn handle_tree(
    content: &Vec<(ObjectEntry, Vec<u8>)>,
    git_dir: &str,
    i: usize,
    path_dir_cloned: &Path,
) -> Result<usize, CommandsError> {
    let tree_content = read_tree(&content[i].1)?;
    builder_object_tree(git_dir, &tree_content)?;
    let i = recovery_tree(tree_content, path_dir_cloned, content, i, git_dir)?;
    Ok(i)
}
