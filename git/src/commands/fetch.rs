use crate::commands::config::GitConfig;
use crate::consts::GIT_DIR;
use crate::git_transport::negotiation::packfile_negotiation_partial;
use crate::models::client::Client;
use crate::util::connections::receive_packfile;
use crate::util::formats::{compressor_object, hash_generate};
use crate::util::objects::{builder_object, ObjectEntry, ObjectType};
use crate::{
    git_transport::{
        git_request::GitRequest, references::reference_discovery, request_command::RequestCommand,
    },
    util::connections::start_client,
};
use std::net::TcpStream;

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
/// Devuelve un `Result` que contiene `Ok(())` en caso de éxito o un error (GitError) en caso de fallo.
///
/// # Errores
///
/// * Otros errores de `GitError`: Pueden ocurrir errores relacionados con la conexión al servidor Git, la inicialización del socket o el proceso de fetch.
///
pub fn handle_fetch(args: Vec<&str>, client: Client) -> Result<String, CommandsError> {
    if args.len() >= 1 {
        return Err(CommandsError::InvalidArgumentCountFetchError);
    }
    let mut socket = start_client(client.get_address())?;
    git_fetch_all(
        &mut socket,
        client.get_ip(),
        client.get_port(),
        client.get_directory_path(),
    )
}

pub fn git_fetch_all(
    socket: &mut TcpStream,
    ip: &str,
    port: &str,
    repo: &str,
) -> Result<String, CommandsError> {
    // Obtengo el repositorio remoto
    let git_config = GitConfig::new_from_repo(repo)?;
    let repo_remoto = git_config.get_remote_repo()?;

    println!("Fetch del repositorio remoto: {}", repo_remoto);

    // Prepara la solicitud "git-upload-pack" para el servidor
    let message =
        GitRequest::generate_request_string(RequestCommand::UploadPack, repo_remoto, ip, port);

    // Reference Discovery
    let mut server = reference_discovery(socket, message)?;

    // Packfile Negotiation
    packfile_negotiation_partial(socket, &mut server, &repo)?;

    // Packfile Data
    let content = receive_packfile(socket)?;
    save_objects(repo, content)?;

    // Guardar las referencias en remote refs

    // Crear archivo FETCH_HEAD

    Ok("Sucessfully!".to_string())
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
fn save_objects(repo: &str, content: Vec<(ObjectEntry, Vec<u8>)>) -> Result<(), CommandsError> {
    let git_dir = format!("{}/{}", repo, GIT_DIR);

    // Guardar los objects
    for object in content.iter() {
        if object.0.obj_type == ObjectType::Commit {
            let commit_hash = hash_generate(&String::from_utf8_lossy(&object.1));
            let file = match builder_object(&git_dir, &commit_hash) {
                Ok(file) => file,
                Err(_) => return Err(CommandsError::RepositoryNotInitialized),
            };
            if compressor_object(String::from_utf8_lossy(&object.1).to_string(), file).is_err() {
                return Err(CommandsError::RepositoryNotInitialized);
            };
        }
    }
    Ok(())
}

// /// Recupera las referencias y objetos del repositorio remoto.
// /// ###Parámetros:
// /// 'directory': directorio del repositorio local.
// /// 'remote_name': nombre del repositorio remoto.
// pub fn git_fetch(directory: &str, remote_name: &str) -> Result<(), GitError> {
//     // Verifica si el repositorio remoto existe
//     let remote_dir = format!("{}{}", REMOTES_DIR, remote_name);
//     let remote_refs_dir = format!("{}{}", directory, remote_dir);

//     if !Path::new(&remote_refs_dir).exists() {
//         return Err(GitError::RemoteDoesntExistError);
//     }

//     // Copia las referencias del repositorio remoto al directorio local
//     let local_refs_dir = format!("{}{}", directory, GIT_DIR);
//     let local_refs_dir = Path::new(&local_refs_dir)
//         .join("refs/remotes")
//         .join(remote_name);

//     if fs::create_dir_all(&local_refs_dir).is_err() {
//         return Err(GitError::OpenFileError);
//     }

//     let entries = match fs::read_dir(&remote_refs_dir) {
//         Ok(entries) => entries,
//         Err(_) => return Err(GitError::ReadFileError),
//     };

//     for entry in entries {
//         match entry {
//             Ok(entry) => {
//                 let file_name = entry.file_name();
//                 let local_ref_path = local_refs_dir.join(file_name);
//                 let remote_ref_path = entry.path();

//                 if fs::copy(remote_ref_path, local_ref_path).is_err() {
//                     return Err(GitError::CopyFileError);
//                 }
//             }
//             Err(_) => {
//                 return Err(GitError::ReadFileError);
//             }
//         }
//     }

//     // Descarga los objetos necesarios desde el repositorio remoto
//     let objects_dir = format!("{}/{}", directory, GIT_DIR);

//     let objects = match fs::read_dir(&objects_dir) {
//         Ok(objects) => objects,
//         Err(_) => return Err(GitError::ReadFileError),
//     };

//     for entry in objects {
//         match entry {
//             Ok(entry) => {
//                 let file_name = entry.file_name();
//                 let object_hash = file_name.to_string_lossy().to_string();

//                 git_cat_file(directory, &object_hash, "-p")?;
//             }
//             Err(_) => {
//                 return Err(GitError::ReadFileError);
//             }
//         }
//     }

//     Ok(())
// }
