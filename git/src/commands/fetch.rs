use crate::commands::branch::get_branch_current_hash;
use crate::commands::config::GitConfig;
use crate::commands::fetch_head::FetchHead;
use crate::consts::{CAPABILITIES_FETCH, GIT_DIR};
use crate::git_server::GitServer;
use crate::git_transport::git_request::GitRequest;
use crate::git_transport::negotiation::packfile_negotiation_partial;
use crate::git_transport::references::{reference_discovery, Reference};
use crate::git_transport::request_command::RequestCommand;
use crate::models::client::Client;
use crate::util::connections::{receive_packfile, send_flush, start_client};
use crate::util::errors::UtilError;
use crate::util::files::create_directory;
use crate::util::objects::{
    builder_object_blob, builder_object_commit, builder_object_tree, read_blob, read_commit,
    read_tree, ObjectEntry, ObjectType,
};
use crate::util::pkt_line::read_pkt_line;
use std::net::TcpStream;
use std::path::Path;
use std::{fmt, fs};

use super::branch::get_branch_remote;
use super::errors::CommandsError;
use super::log::save_log;

#[derive(Debug)]
pub enum FetchStatus {
    // Success(String),
    NoUpdatesRemote(String),
    NoUpdatesBranch(String),
    UpdatesBranch(String),
    BranchNotFound(String),
    BranchHasNoExistingCommits(String),
    SomeRemotesUpdated(String),
}

impl fmt::Display for FetchStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // FetchStatus::Success(String) => write!(f, "El fetch se completó exitosamente. Se recuperaron nuevas actualizaciones."),
            FetchStatus::NoUpdatesRemote(s) => write!(f, "No hay nuevas actualizaciones en el repositorio remoto: {}. Todo está actualizado.", s),
            FetchStatus::NoUpdatesBranch(s) => write!(f, "No hay nuevas actualizaciones en la branch: {}. Todo está actualizado.", s),
            FetchStatus::UpdatesBranch(s) => write!(f, "Branch objects updated:\n{}", s),
            FetchStatus::BranchNotFound(s) => write!(f, "La branch: {}\nNo existe en el repositorio remoto. Haga push", s),
            FetchStatus::BranchHasNoExistingCommits(s) => write!(f, "La branch: {}\nNo tiene commits. Realice add y commit", s),
            FetchStatus::SomeRemotesUpdated(s) => write!(f, "Se actualizaron las siguientes branch:\n{}", s),
        }
    }
}

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
pub fn handle_fetch(args: Vec<&str>, client: Client) -> Result<FetchStatus, CommandsError> {
    if args.len() >= 3 || args.len() == 1 {
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
        args[1],
    )
}

pub fn git_fetch_all(
    socket: &mut TcpStream,
    ip: &str,
    port: &str,
    repo_local: &str,
) -> Result<FetchStatus, CommandsError> {
    println!("Estoy aqui");
    // Obtengo los remotos en uso
    let git_config = GitConfig::new_from_file(repo_local)?;
    let remotes = git_config.get_remotes_in_use();
    let mut status = Vec::new();
    println!("Remotes: {:?}", remotes);

    for name_remote in remotes {
        let url_remote = &git_config.get_remote_url_by_name(&name_remote)?;
        let status_remote = _git_fetch_all(socket, ip, port, repo_local, url_remote, &name_remote)?;
        status.push(status_remote.to_string());
    }

    Ok(FetchStatus::SomeRemotesUpdated(status.join("\n")))
}

pub fn _git_fetch_all(
    socket: &mut TcpStream,
    ip: &str,
    port: &str,
    repo_local: &str,
    url_remote: &str,
    remote_branch: &str,
) -> Result<FetchStatus, CommandsError> {
    // Obtengo el repositorio remoto
    println!("Repositorio local: {}", repo_local);
    println!("Fetch del repositorio remoto: {}", url_remote);

    // Prepara la solicitud "git-upload-pack" para el servidor
    let message =
        GitRequest::generate_request_string(RequestCommand::UploadPack, url_remote, ip, port);

    // Reference Discovery
    let my_capacibilities: Vec<String> =
        CAPABILITIES_FETCH.iter().map(|&s| s.to_string()).collect();
    let mut server = reference_discovery(socket, message, url_remote, &my_capacibilities)?;
    println!("Reference Discovery");

    // Packfile Negotiation
    packfile_negotiation_partial(socket, &mut server, repo_local)?;
    println!("packfile_negotiation_partial");

    // Packfile Data
    let _last_ack = read_pkt_line(socket)?; // Vlidar last ack
    println!("Recibi el ultimo ack");
    println!("_last_ack: {:?}", _last_ack);

    let content = receive_packfile(socket)?;
    for (object, _) in &content {
        println!("FETCH --- > object: {:?}", object);
        // println!("bytes: {:?}", bytes);
    }
    if content.is_empty() {
        println!("No hay actualizaciones");
        return Ok(FetchStatus::NoUpdatesRemote(url_remote.to_string()));
    }
    println!("receive_packfile --> {:?}", content);

    let refs = server.get_references_for_updating()?;

    if !is_already_update(repo_local, &refs, remote_branch)? {
        if save_objects(content, repo_local).is_err() {
            return Err(CommandsError::RepositoryNotInitialized);
        };
        save_references(&refs, repo_local, remote_branch)?;
        let mut fetch_head = FetchHead::new_from_file(repo_local)?;
        fetch_head.update_references(&refs, url_remote)?;
        fetch_head.write(repo_local)?;
        let mut status = Vec::new();
        for reference in refs {
            status.push(format!(
                "New update: {} --> {}, Can merge",
                reference.get_ref_path(),
                reference.get_hash()
            ));
        }
        Ok(FetchStatus::UpdatesBranch(status.join("\n")))
    } else {
        Ok(FetchStatus::NoUpdatesRemote(url_remote.to_string()))
    }
}

pub fn git_fetch_branch(
    socket: &mut TcpStream,
    ip: &str,
    port: &str,
    repo_local: &str,
    name_remote: &str,
    name_branch: &str,
) -> Result<FetchStatus, CommandsError> {
    // Obtengo el repositorio remoto
    println!("Repositorio local: {}", repo_local);
    let git_config = GitConfig::new_from_file(repo_local)?;
    println!("Config: {:?}", git_config);

    let url_remoto = &git_config.get_remote_url_by_name(name_remote)?;
    println!("url: {}", url_remoto);

    println!("Fetch del repositorio remoto: {}", url_remoto);

    let rfs_fetch = format!("refs/heads/{}", name_branch);

    // Prepara la solicitud "git-upload-pack" para el servidor
    let message =
        GitRequest::generate_request_string(RequestCommand::UploadPack, url_remoto, ip, port);

    // Reference Discovery
    let my_capacibilities: Vec<String> =
        CAPABILITIES_FETCH.iter().map(|&s| s.to_string()).collect();
    let mut server = reference_discovery(socket, message, url_remoto, &my_capacibilities)?;
    if !server.contains_reference(&rfs_fetch) {
        send_flush(socket, UtilError::SendFlushCancelConnection)?;
        return Ok(FetchStatus::BranchNotFound(name_branch.to_string()));
    }

    // Packfile Negotiation
    // Solo solicitar una branch
    server.update_references_filtering([rfs_fetch].to_vec())?;
    packfile_negotiation_partial(socket, &mut server, repo_local)?;

    // Packfile Data
    let _last_ack = read_pkt_line(socket)?; // Vlidar last ack
    let content = receive_packfile(socket)?;

    if content.is_empty() {
        return Ok(FetchStatus::NoUpdatesBranch(name_branch.to_string()));
    }

    let refs = server.get_references_for_updating()?;
    println!("Refs: {:?}", refs);

    if !is_already_update(repo_local, &refs, name_branch)? {
        if save_objects(content, repo_local).is_err() {
            println!("Error al guardar los objetos");
            return Err(CommandsError::RepositoryNotInitialized);
        };
        save_references(&refs, repo_local, name_remote)?;

        let mut fetch_head = FetchHead::new_from_file(repo_local)?;
        fetch_head.update_references(&refs, url_remoto)?;
        fetch_head.write(repo_local)?;

        let mut status = Vec::new();
        for reference in refs {
            status.push(format!(
                "New update: {} --> {}, Can merge",
                reference.get_ref_path(),
                reference.get_hash()
            ));
        }
        Ok(FetchStatus::UpdatesBranch(status.join("\n")))
    } else {
        Ok(FetchStatus::NoUpdatesBranch(name_branch.to_string()))
    }

    // Ok(FetchStatus::SomeRemotesUpdated(format!("{} --> {}", name_branch, refs.)))
}

/// Recibe las referencias del servidor y las compara los hashes de cada branch con el repositorio local,
/// en caso de ser todas iguales devuelve true, sino false.
///  
/// # Argumentos
///
/// * `repo_local`: Directorio del repositorio local
/// * `refs`: Referencias del servidor
/// * `name_remote`: Nombre del repositorio remoto
///
/// # Errores
///
/// Devuelve un error del tipo `CommandsError` si hay problemas
///
fn is_already_update(
    repo_local: &str,
    refs: &Vec<Reference>,
    name_remote: &str,
) -> Result<bool, CommandsError> {
    let mut found = false;
    let branches = match get_branch_remote(repo_local, name_remote) {
        Ok(branches) => branches,
        Err(CommandsError::BranchDirectoryOpenError) => return Ok(false),
        Err(e) => return Err(e),
    };

    if branches.is_empty() {
        return Ok(false);
    }
    for reference in refs {
        for branch in branches.clone() {
            if *reference.get_name() == branch {
                found = true;
            }
        }
    }
    if !found {
        return Ok(false);
    }
    for branch in branches {
        for reference in refs {
            let local_branch_hash = get_branch_current_hash(repo_local, branch.clone())?;
            let ref_branch_hash = String::from(reference.get_hash());
            if branch == reference.get_name() && local_branch_hash != ref_branch_hash {
                return Ok(false);
            }
        }
    }
    Ok(true)
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
pub fn get_branches_remote(server: &GitServer) -> Result<Vec<(String, String)>, CommandsError> {
    let mut references: Vec<(String, String)> = vec![];

    for reference in server.get_references().iter().skip(1) {
        let hash = reference.get_hash();
        let branch = reference.get_ref_path();
        if let Some(current_branch) = branch.rsplit('/').next() {
            let new_ref: (String, String) = (current_branch.to_string(), hash.to_string());
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
pub fn save_references(
    references: &Vec<Reference>,
    repo_path: &str,
    name_remote: &str,
) -> Result<(), CommandsError> {
    // Si no existe el directorio .git/refs/remotes lo crea
    let directory_remotes = format!("{}/.git/refs/remotes", repo_path);
    let directory_remotes = Path::new(&directory_remotes);
    create_directory(directory_remotes)?;

    // Si no existe el directorio .git/refs/remotes/origin lo crea
    let refs_dir_path = format!("{}/.git/refs/remotes/{}", repo_path, name_remote);
    //ensure_directory_clean(&refs_dir_path)?;  NO VA?

    let log_dir = format!("{}/.git/logs/refs/remotes/{}", repo_path, name_remote);
    create_directory(Path::new(&log_dir))?;

    // Escribe los hashes en archivos individuales
    for reference in references {
        let name_branch = reference.get_name();
        let hash = reference.get_hash();
        let file_path = format!("{}/{}", refs_dir_path, name_branch);
        if fs::write(&file_path, hash).is_err() {
            return Err(CommandsError::RemotoNotInitialized);
        };
        let path_log = format!("logs/refs/remotes/{}", name_remote);

        let path_branch = format!("refs/remotes/{}", name_remote);

        save_log(repo_path, name_branch, &path_log, &path_branch)?;
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
pub fn save_objects(
    content: Vec<(ObjectEntry, Vec<u8>)>,
    git_dir: &str,
) -> Result<(), CommandsError> {
    // Cantidad de objetos recibidos
    let count_objects = content.len();
    let git_dir = format!("{}/{}", git_dir, GIT_DIR);

    let mut i = 0;
    while i < count_objects {
        if content[i].0.obj_type == ObjectType::Commit {
            let commit_content = read_commit(&content[i].1)?;
            builder_object_commit(&commit_content, &git_dir)?;
            i += 1;
        } else if content[i].0.obj_type == ObjectType::Tree {
            let tree_content = read_tree(&content[i].1)?;
            builder_object_tree(&git_dir, &tree_content)?;
            i += 1;
        } else if content[i].0.obj_type == ObjectType::Blob {
            let blob_content = read_blob(&content[i].1)?;
            builder_object_blob(blob_content.into_bytes(), &git_dir)?;
            i += 1;
        }
    }
    Ok(())
}
