use crate::models::client::Client;
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
pub fn handle_fetch(args: Vec<&str>, client: Client) -> Result<(), CommandsError> {
    if !args.is_empty() {
        return Err(CommandsError::InvalidArgumentCountFetchError);
    }
    let mut socket = start_client(client.get_address())?;
    git_fetch_all(&mut socket, client.get_ip(), client.get_port(), args[0])
}

pub fn git_fetch_all(
    socket: &mut TcpStream,
    ip: &str,
    port: &str,
    repo: &str,
) -> Result<(), CommandsError> {
    println!("Fetch del repositorio remoto: {}", repo);

    // Prepara la solicitud "git-upload-pack" para el servidor
    let message = GitRequest::generate_request_string(RequestCommand::UploadPack, repo, ip, port);

    // Reference Discovery
    let _advertised = reference_discovery(socket, message)?;

    // println!("Fetching from remote repository: {}", directory);
    // println!("Fetching references...");
    // println!("ip: {}", ip);
    // println!("port: {}", port);

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
