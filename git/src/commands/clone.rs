use crate::commands::commit::builder_commit_log;
use crate::commands::config::GitConfig;
use crate::commands::init::git_init;
use crate::consts::{DIRECTORY, FILE, GIT_DIR, REF_HEADS, PARENT_INITIAL};
use crate::util::validation::join_paths_correctly;
use super::errors::CommandsError;
use crate::git_server::GitServer;
use crate::git_transport::git_request::GitRequest;
use crate::git_transport::references::reference_discovery;
use crate::git_transport::request_command::RequestCommand;
use crate::models::client::Client;
use crate::util::connections::{packfile_negotiation, receive_packfile, start_client};
use crate::util::files::{create_directory, create_file, create_file_replace};
use crate::util::objects::{
    builder_object_blob, builder_object_commit, builder_object_tree, read_blob, read_commit,
    read_tree,
};
use crate::util::objects::{ObjectEntry, ObjectType};
use std::net::TcpStream;
use std::path::Path;

use super::add::add_to_index;

/// Maneja la ejecución del comando "clone" en el cliente Git.
///
/// # Developer
///
/// Solo se aceptaran los comandos que tengan la siguiente estructura:
///
/// * `git clone <path_name>`
///
/// # Argumentos
///
/// * `args`: Un vector que contiene los argumentos pasados al comando "clone". Se espera que tenga exactamente un elemento, que es la URL del repositorio Git que se va a clonar.
///
/// * `client`: Un objeto `Client` que representa la configuración del cliente Git.
///
/// # Devoluciones
///
/// Devuelve un `Result` que contiene un mensaje de éxito (String) o un error (CommandsError).
///
/// # Errores
///
/// * `CloneMissingRepoError`: Se produce si no se proporciona la URL del repositorio Git para clonar.
///
/// * Otros errores de `CommandsError`: Pueden ocurrir errores relacionados con la conexión al servidor Git, la inicialización del socket, o el proceso de clonación.
///
pub fn handle_clone(args: Vec<&str>, client: Client) -> Result<(String, String), CommandsError> {
    if args.len() != 1 {
        return Err(CommandsError::CloneMissingRepoError.into());
    }
    let mut socket = start_client(client.get_address())?;
    let name = match args[0].split('/').last()
    {
        Some(name) => name,
        None => return Err(CommandsError::CloneMissingRepoError),
    };
    let local_repo = join_paths_correctly(client.get_directory_path(), name);
    println!("local repo: {}", local_repo);
    println!("client: {:?}", client);
    git_clone(&mut socket, client.get_ip(), client.get_port(), &local_repo, args[0])
}

/// Clona un repositorio Git desde un servidor remoto utilizando el protocolo Git.
///
/// # Argumentos
///
/// - `socket`: Una referencia mutable a un `TcpStream` que representa la conexión con el servidor.
/// - `ip`: La dirección IP del servidor Git.
/// - `port`: El número de puerto utilizado para la conexión.
/// - `name_repo`: El nombre del repositorio Git que se va a clonar.
/// - `path_repo`: La ruta del repositorio Git que se va a clonar.
///
/// # Returns
///
/// Un `Result` que contiene una cadena indicando el éxito del clon o un error `CommandsError` en caso de error.
///
pub fn git_clone(
    socket: &mut TcpStream,
    ip: &str,
    port: &str,
    local_repo: &str,
    remote_repo: &str,
) -> Result<(String, String), CommandsError> {
    println!("Clonando repositorio remoto: {}", remote_repo);
    println!("En el directorio: {}", local_repo);

    // Prepara la solicitud "git-upload-pack" para el servidor
    let message = GitRequest::generate_request_string(RequestCommand::UploadPack, remote_repo, ip, port);

    // Reference Discovery
    let git_server = reference_discovery(socket, message, remote_repo)?;

    // Packfile Negotiation
    packfile_negotiation(socket, &git_server)?;

    // Packfile Data
    let content = receive_packfile(socket)?;

    let status = create_repository(&git_server, content, local_repo)?;

    // Creo el config
    let git_config = GitConfig::new_from_server(&git_server)?;
    let path_config = format!("{}/{}/{}", local_repo, GIT_DIR, "config");
    git_config.write_to_file(&path_config)?;


    Ok((status, local_repo.to_string()))
}

fn create_repository(
    advertised: &GitServer,
    content: Vec<(ObjectEntry, Vec<u8>)>,
    repo: &str,
) -> Result<String, CommandsError> {
    // Cantidad de objetos recibidos
    let count_objects = content.len();

    let path_dir_cloned = Path::new(repo);
    git_init(repo)?;
    let git_dir = format!("{}/{}", repo, GIT_DIR);

    let mut i = 0;
    while i < count_objects {
        if content[i].0.obj_type == ObjectType::Commit {
            handle_commit(&content, repo, advertised, &git_dir, i)?;
            i += 1;
        } else if content[i].0.obj_type == ObjectType::Tree {
            i = match handle_tree(&content, &git_dir, i, path_dir_cloned) {
                Ok(i) => i,
                Err(e) => return Err(e),
            };
            i += 1;
        }
    }
    Ok("Clonación exitosa!".to_string())
}

fn recovery_blob(
    hash: &str,
    path_dir_cloned: &Path,
    content: &Vec<(crate::util::objects::ObjectEntry, Vec<u8>)>,
    mut i: usize,
    repo: &str,
) -> Result<usize, CommandsError> {
    if i < content.len(){
        let route: Vec<_> = path_dir_cloned.components().skip(1)
        .map(|c| c.as_os_str().to_str())
        .filter_map(|s| s)
        .collect();
        let blob_content = read_blob(&content[i].1)?;
        let blob_content_bytes = blob_content.clone();
        if !path_dir_cloned.exists(){
            add_to_index(repo.to_string(), &route.join("/"), hash.to_string())?;
            builder_object_blob(blob_content_bytes.into_bytes(), repo)?;
            if let Some(str_path) = path_dir_cloned.to_str() {
                create_file_replace(str_path, &blob_content)?;
            }
        }else{
            i -= 1;
        }
    }
    Ok(i)
}

fn recovery_tree(
    tree_content: String,
    path_dir_cloned: &Path,
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
        
        let path_dir_cloned = path_dir_cloned.join(file_name);
        if mode == FILE {
            i += 1;
            i = recovery_blob(hash, &path_dir_cloned, content, i, repo)?;
            
        } else if mode == DIRECTORY {
            i += 1;
            if i < content.len(){
                create_directory(&path_dir_cloned)?;
                let tree_content = read_tree(&content[i].1)?;
                builder_object_tree(repo, &tree_content)?;
                i = recovery_tree(tree_content, &path_dir_cloned, content, i, repo)?;
            }
        }
    }
    Ok(i)
}

fn insert_line_between_lines(
    original_string: &str,
    line_number_1: usize,
    new_line: &str,
) -> String {
    let mut result = String::new();

    let lines = original_string.lines();

    for (index, line) in lines.enumerate() {
        result.push_str(line);
        result.push('\n');
        if index + 1 == line_number_1 {
            let parent_format = format!("parent {}", new_line);
            result.push_str(&parent_format);
            result.push('\n');
        }
    }

    result
}


fn handle_commit(
    content: &[(ObjectEntry, Vec<u8>)],
    repo: &str,
    advertised: &GitServer,
    git_dir: &str,
    i: usize,
) -> Result<(), CommandsError> {
    let mut commit_content = read_commit(&content[i].1)?;
    
    builder_object_commit(&commit_content, git_dir)?;

    if let Some(refs) = advertised.get_reference(i + 1) {
        let hash = refs.get_hash();
        let branch = refs.get_name();

        if let Some(current_branch) = branch.rsplit('/').next() {
            let branch_dir = format!("{}/{}/{}/{}", repo, GIT_DIR, REF_HEADS, current_branch);
            create_file(&branch_dir, hash)?;
        }
        if commit_content.lines().count() == 5{
            commit_content = insert_line_between_lines(&commit_content, 1, PARENT_INITIAL);
        }
        builder_commit_log(repo, &commit_content, hash)?;
    }

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
