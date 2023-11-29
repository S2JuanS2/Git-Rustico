use crate::commands::config::GitConfig;
use crate::commands::fetch_head::FetchHead;
use crate::consts::{DIRECTORY, FILE, GIT_DIR, CAPABILITIES_FETCH};
use crate::git_server::GitServer;
use crate::git_transport::negotiation::packfile_negotiation_partial;
use crate::git_transport::references::{reference_discovery, Reference};
use crate::git_transport::request_command::RequestCommand;
use crate::models::client::Client;
use crate::util::connections::{receive_packfile, start_client};
use crate::util::files::ensure_directory_clean;
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
    // println!("server: {:?}", server);
    // Packfile Negotiation
    packfile_negotiation_partial(socket, &mut server, repo_local)?;

    // Packfile Data
    let last_ack = read_pkt_line(socket)?;
    println!("last_ack: {}", String::from_utf8_lossy(&last_ack));
    let content = receive_packfile(socket)?;
    for c in &content
    {
        println!("ObjectEntry: {:?} --- Content: {:?}", c.0, c.1);
        // println!("")
    }
    if save_objects(content, repo_local).is_err() {
        return Err(CommandsError::RepositoryNotInitialized);
    };

    // // Guardar las referencias en remote refs
    // // [TODO]
    // // necesito una funcion que me devuleva un vector dek tipo Vec<(String, String)>
    // // EL 1er string sera el nombre de la branch y el 2do string su ultimo commit
    // // Se puede usar el content o otro objeto
    // // let refs: Vec<(String, String)> = get_refs(content);
    // let refs = get_branches(&server)?;
    // save_references(&refs, repo_local)?;

    let refs = server.get_references_for_updating()?;
    // Guardo las referencias
    save_references(&refs, repo_local)?;
    // Crear archivo FETCH_HEAD
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
    
    let refs_dir_path = format!("{}/.git/refs/origin", repo_path);

    // Crea el directorio si no existe
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
    }
    Ok(())
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

        let mode = parts[0];
        let file_name = parts[1];
        let _hash = parts[2];

        let path_dir_repo = path_dir_repo.join(file_name);
        if mode == FILE {
            i += 1;
            let blob_content = read_blob(&content[i].1)?;
            let blob_content_bytes = blob_content.clone();
            builder_object_blob(blob_content_bytes.into_bytes(), repo)?;
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

// /// Recupera las referencias y objetos del repositorio remoto.
// /// ###Parámetros:
// /// 'directory': directorio del repositorio local.
// /// 'remote_name': nombre del repositorio remoto.
// pub fn git_fetch(directory: &str, remote_name: &str) -> Result<(), CommandsError> {
//     // Verifica si el repositorio remoto existe
//     let remote_dir = format!("{}{}", REMOTES_DIR, remote_name);
//     let remote_refs_dir = format!("{}{}", directory, remote_dir);

//     if !Path::new(&remote_refs_dir).exists() {
//         return Err(CommandsError::RemoteDoesntExistError);
//     }

//     // Copia las referencias del repositorio remoto al directorio local
//     let local_refs_dir = format!("{}{}", directory, GIT_DIR);
//     let local_refs_dir = Path::new(&local_refs_dir)
//         .join("refs/remotes")
//         .join(remote_name);

//     if fs::create_dir_all(&local_refs_dir).is_err() {
//         return Err(CommandsError::OpenFileError);
//     }

//     let entries = match fs::read_dir(&remote_refs_dir) {
//         Ok(entries) => entries,
//         Err(_) => return Err(CommandsError::ReadFileError),
//     };

//     for entry in entries {
//         match entry {
//             Ok(entry) => {
//                 let file_name = entry.file_name();
//                 let local_ref_path = local_refs_dir.join(file_name);
//                 let remote_ref_path = entry.path();

//                 if fs::copy(remote_ref_path, local_ref_path).is_err() {
//                     return Err(CommandsError::CopyFileError);
//                 }
//             }
//             Err(_) => {
//                 return Err(CommandsError::ReadFileError);
//             }
//         }
//     }

//     // Descarga los objetos necesarios desde el repositorio remoto
//     let objects_dir = format!("{}/{}", directory, GIT_DIR);

//     let objects = match fs::read_dir(&objects_dir) {
//         Ok(objects) => objects,
//         Err(_) => return Err(CommandsError::ReadFileError),
//     };

//     for entry in objects {
//         match entry {
//             Ok(entry) => {
//                 let file_name = entry.file_name();
//                 let object_hash = file_name.to_string_lossy().to_string();

//                 git_cat_file(directory, &object_hash, "-p")?;
//             }
//             Err(_) => {
//                 return Err(CommandsError::ReadFileError);
//             }
//         }
//     }

//     Ok(())
// }
