use crate::commands::fetch::{git_fetch_branch, FetchStatus};
use crate::commands::fetch_head::FetchHead;
use crate::git_transport::references::Reference;
use super::errors::CommandsError;
use crate::models::client::Client;
use crate::util::connections::start_client;
use std::net::TcpStream;

/// Maneja el comando "pull".
///
/// Esta función inicia una operación de pull desde el servidor de Git.
///
/// # Argumentos
///
/// * `args` - Un vector de argumentos de línea de comandos. Debería estar vacío para una operación de pull.
/// * `client` - Una instancia de la estructura `Client` que contiene detalles de la conexión.
///
/// # Devuelve
///
/// * `Result<(), CommandsError>` - Un resultado que indica el éxito o un error encontrado durante la operación de pull.
///
/// # Errores
///
/// * `CommandsError::InvalidArgumentCountPull` - Indica que se proporcionó un número incorrecto de argumentos para el comando pull.
/// * `CommandsError` - Indica varios errores relacionados con Git que podrían ocurrir durante la operación de pull.
///
pub fn handle_pull(args: Vec<&str>, client: Client) -> Result<String, CommandsError> {
    if !args.is_empty() {
        return Err(CommandsError::InvalidArgumentCountPull);
    }
    let mut socket = start_client(client.get_address())?;
    git_pull(
        &mut socket,
        client.get_ip(),
        client.get_port(),
        client.get_directory_path(),
    )
}

// Quiero actualizar mi branch actual con los cambios del repositorio remoto
pub fn git_pull(
    socket: &mut TcpStream,
    ip: &str,
    port: &str,
    repo_local: &str,
) -> Result<String, CommandsError> {
    // Obtengo el repositorio remoto
    println!("Pull del repositorio remoto ...");

    let current_rfs = match Reference::get_current_references(repo_local)
    {
        Ok(rfs) => rfs,
        Err(_) => return Err(CommandsError::PullCurrentBranchNotFound),
    };
    let name_branch = current_rfs.get_name();
    
    let result =  git_fetch_branch(socket, ip, port, repo_local, &name_branch)?;
    println!("Result: {:?}", result);
    match result {
        FetchStatus::BranchNotFound => return Ok(format!("{}", FetchStatus::BranchNotFound)),
        FetchStatus::NoUpdates => return Ok(format!("{}", FetchStatus::NoUpdates)),
        FetchStatus::BranchHasNoExistingCommits => return Ok(format!("{}", FetchStatus::BranchHasNoExistingCommits)),
        FetchStatus::Success => (),
    }

    // Esto pasa cuando ya hicimos fetch anteriormente y no mergeamos
    let mut fetch_head = FetchHead::new_from_file(repo_local)?;
    if !fetch_head.references_needs_update(current_rfs.get_name())
    {
        return Ok(format!("{}", FetchStatus::NoUpdates));
    }

    
    let _update_rfs = fetch_head.get_references(current_rfs.get_name())?;
    
    // [TODO #6]
    // Dado current references y update references, hacer merge
    // Datos:
    // current_rfs.get_hash() -> hash del commit actual
    // update_rfs.get_hash() -> hash del commit remoto
    // current_rfs.get_name() -> nombre de la branch actual
    // update_rfs.get_name() -> nombre de la branch remota
    // current_rfs.get_path() -> path del archivo de la branch actual
    // update_rfs.get_path() -> path del archivo de la branch remota
    // repo_local -> path del repo local
    // [FIN TODO #6]
    

    // Actualizo el fetch_head si todo salio bien
    fetch_head.delete_references(current_rfs.get_name())?;
    fetch_head.write(repo_local)?;

    Ok("Pullcito naciendo".to_string())
}


// Err(_) => return Err(CommandsError::PullCurrentBranchNotFound),
