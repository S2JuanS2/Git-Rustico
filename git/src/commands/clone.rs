use super::errors::CommandsError;
use super::log::save_log;
use crate::commands::config::GitConfig;
use crate::commands::init::git_init;
use crate::consts::{DIRECTORY, FILE, GIT_DIR, REF_HEADS};
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
use crate::util::validation::join_paths_correctly;
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
        return Err(CommandsError::CloneMissingRepoError);
    }
    let mut socket = start_client(client.get_address())?;
    let name = match args[0].split('/').last() {
        Some(name) => name,
        None => return Err(CommandsError::CloneMissingRepoError),
    };
    let local_repo = join_paths_correctly(client.get_directory_path(), name);
    println!("local repo: {}", local_repo);
    println!("client: {:?}", client);
    git_clone(
        &mut socket,
        client.get_ip(),
        client.get_port(),
        &local_repo,
        args[0],
    )
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
    let message =
        GitRequest::generate_request_string(RequestCommand::UploadPack, remote_repo, ip, port);

    // Reference Discovery
    let git_server = reference_discovery(socket, message, remote_repo, &Vec::new())?;

    // Packfile Negotiation
    packfile_negotiation(socket, &git_server)?;

    // Packfile Data
    let content = receive_packfile(socket)?;

    let local_repo_parts: Vec<&str> = local_repo.split('/').collect();
    let status = create_repository(content, local_repo, local_repo_parts.len())?;
    save_references(&git_server, local_repo)?;

    // Creo el config
    let git_config = GitConfig::new_from_server(&git_server)?;
    let path_config = format!("{}/{}/{}", local_repo, GIT_DIR, "config");
    git_config.write_to_file(&path_config)?;

    Ok((status, local_repo.to_string()))
}

/// Crea un repositorio a partir de los objetos recibidos del servidor.
///
/// # Argumentos
///
/// - `content`: Objetos recibidos desde el servidor
/// - `repo`: Dirección del repositorio del clone
/// - `repo_count`: Cantidad de objetos a crear
///
/// # Returns
///
/// Un `Result` que contiene una cadena indicando el éxito del clone o un error `CommandsError` en caso de error.
///
fn create_repository(
    content: Vec<(ObjectEntry, Vec<u8>)>,
    repo: &str,
    repo_count: usize,
) -> Result<String, CommandsError> {
    // Cantidad de objetos recibidos
    let count_objects = content.len();

    let path_dir_cloned = Path::new(repo);
    git_init(repo)?;
    let git_dir = format!("{}/{}", repo, GIT_DIR);
    let mut first_tree = 0;
    let mut i = 0;
    while i < count_objects {
        if content[i].0.obj_type == ObjectType::Commit {
            handle_commit(&content, &git_dir, i)?;
            i += 1;
        } else if content[i].0.obj_type == ObjectType::Tree {
            i = match handle_tree(
                &content,
                &git_dir,
                i,
                path_dir_cloned,
                repo_count,
                first_tree,
            ) {
                Ok(i) => i,
                Err(e) => return Err(e),
            };
            i += 1;
            first_tree = 1;
        } else if content[i].0.obj_type == ObjectType::Blob {
            i += 1;
        }
    }
    Ok("Successful cloning".to_string())
}

/// Construye el blob y lo agrega al index.
///
/// # Argumentos
///
/// - `hash`: hash del blob
/// - `path_dir_cloned`: Dirección del blob
/// - `content`: Objetos recibidos desde el servidor
/// - `repo_count`: Cantidad de objetos a crear
/// - `repo`: Repositorio del cliente
///
/// # Returns
///
/// Un `Result` que contiene el numero de objeto actual o un error `CommandsError` en caso de error.
///
fn recovery_blob(
    hash: &str,
    path_dir_cloned: &Path,
    content: &[(crate::util::objects::ObjectEntry, Vec<u8>)],
    mut i: usize,
    repo: &str,
    repo_count: usize,
    first_tree: usize,
) -> Result<usize, CommandsError> {
    if i < content.len() {
        let route: Vec<_> = path_dir_cloned
            .components()
            .skip(repo_count)
            .filter_map(|c| c.as_os_str().to_str())
            .collect();
        let blob_content = read_blob(&content[i].1)?;
        let blob_content_bytes = blob_content.clone();
        if !path_dir_cloned.exists() {
            println!("{:?}", route);
            builder_object_blob(blob_content_bytes.into_bytes(), repo)?;
            if let Some(str_path) = path_dir_cloned.to_str() {
                if first_tree == 0 {
                    add_to_index(repo.to_string(), &route.join("/"), hash.to_string())?;
                    create_file_replace(str_path, &blob_content)?;
                }
            }
        } else {
            i -= 1;
        }
    }
    Ok(i)
}

/// Recorre los objetos sub-tree, lo construye y lo agrega al index.
///
/// # Argumentos
///
/// - `tree_content`: Contenido del tree
/// - `path_dir_cloned`: Dirección del tree
/// - `content`: Objetos recibidos desde el servidor
/// - `repo_count`: Cantidad de objetos a crear
/// - `repo`: Repositorio del cliente
///
/// # Returns
///
/// Un `Result` que contiene el numero de objeto actual o un error `CommandsError` en caso de error.
///
fn recovery_tree(
    tree_content: String,
    path_dir_cloned: &Path,
    content: &Vec<(crate::util::objects::ObjectEntry, Vec<u8>)>,
    mut i: usize,
    repo: &str,
    repo_count: usize,
    first_tree: usize,
) -> Result<usize, CommandsError> {
    for line in tree_content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 3 {
            let mode;
            let file_name;
            if parts[0] == FILE || parts[0] == DIRECTORY {
                mode = parts[0];
                file_name = parts[1];
            } else {
                file_name = parts[0];
                mode = parts[1];
            }
            let hash = parts[2];

            let path_dir_cloned = path_dir_cloned.join(file_name);
            if mode == FILE {
                i += 1;
                i = recovery_blob(
                    hash,
                    &path_dir_cloned,
                    content,
                    i,
                    repo,
                    repo_count,
                    first_tree,
                )?;
            } else if mode == DIRECTORY {
                i += 1;
                if i < content.len() {
                    if first_tree == 0 {
                        create_directory(&path_dir_cloned)?;
                    }
                    let tree_content = read_tree(&content[i].1)?;
                    builder_object_tree(repo, &tree_content)?;
                    let count = i;
                    i = recovery_tree(
                        tree_content,
                        &path_dir_cloned,
                        content,
                        i,
                        repo,
                        repo_count,
                        first_tree,
                    )?;
                    if count == i {
                        i -= 1;
                    }
                }
            }
        }
    }
    Ok(i)
}

/// Recorre las referencias recibidas del servidor y las guarda en el repositorio local
///
/// # Argumentos
///
/// - `repo`: Dirección del repositorio
/// - `advertised`: Contiene las referencias
///
/// # Returns
///
/// Un `Result` con un retorno `CommandsError` en caso de error.
///
fn save_references(advertised: &GitServer, repo: &str) -> Result<(), CommandsError> {
    //refs/remotes/origin
    for refs in advertised.get_references().iter().skip(1) {
        let hash = refs.get_hash();
        let branch = refs.get_name();
        if let Some(current_branch) = branch.rsplit('/').next() {
            let branch_dir = format!("{}/{}/{}/{}", repo, GIT_DIR, REF_HEADS, current_branch);
            create_file(&branch_dir, hash)?;
            save_log(repo, current_branch, "logs/refs/heads", REF_HEADS)?;
        }
    }
    Ok(())
}

/// Construye el objeto Commit recibido del servidor
///
/// # Argumentos
///
/// - `git_dir`: Dirección del repositorio
/// - `content`: Objetos recibidos del servidor
/// - `i`: Numero de objeto actual
///
/// # Returns
///
/// Un `Result` con un retorno `CommandsError` en caso de error.
///
fn handle_commit(
    content: &[(ObjectEntry, Vec<u8>)],
    git_dir: &str,
    i: usize,
) -> Result<(), CommandsError> {
    let commit_content = read_commit(&content[i].1)?;
    builder_object_commit(&commit_content, git_dir)?;

    Ok(())
}

/// Recorre los objetos tree, lo construye y lo agrega al index.
///
/// # Argumentos
///
/// - `content`: Objetos recibidos desde el servidor
/// - `git_dir`: Repositorio del cliente
/// - `tree_content`: Contenido del tree
/// - `path_dir_cloned`: Dirección del tree
/// - `repo_count`: Cantidad de objetos a crear
///
/// # Returns
///
/// Un `Result` que contiene el numero de objeto actual o un error `CommandsError` en caso de error.
///
fn handle_tree(
    content: &Vec<(ObjectEntry, Vec<u8>)>,
    git_dir: &str,
    i: usize,
    path_dir_cloned: &Path,
    repo_count: usize,
    first_tree: usize,
) -> Result<usize, CommandsError> {
    let tree_content = read_tree(&content[i].1)?;
    builder_object_tree(git_dir, &tree_content)?;
    let i = recovery_tree(
        tree_content,
        path_dir_cloned,
        content,
        i,
        git_dir,
        repo_count,
        first_tree,
    )?;
    Ok(i)
}
