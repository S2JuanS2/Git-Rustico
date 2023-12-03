use crate::commands::config::GitConfig;
use crate::commands::fetch::git_fetch_branch;
use crate::commands::fetch_head::FetchHead;
use crate::commands::merge::{git_merge, get_conflict_path};
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
    let mut status = Vec::new();
    let current_rfs = match Reference::get_current_references(repo_local)
    {
        Ok(rfs) => rfs,
        Err(_) => return Err(CommandsError::PullCurrentBranchNotFound),
    };
    let name_branch = current_rfs.get_name();
    
    let result =  git_fetch_branch(socket, ip, port, repo_local, name_branch)?;
    status.push(format!("{}", result));
    println!("Result del fetch: {}", result);

    // Esto pasa cuando ya hicimos fetch anteriormente y no mergeamos
    let mut fetch_head = FetchHead::new_from_file(repo_local)?;
    if !fetch_head.references_needs_update(current_rfs.get_name())
    {
        status.push("No hay actualizaciones para mergear".to_string());
        return Ok(status.join("\n"));
    }

    let git_config = GitConfig::new_from_file(repo_local)?;
    let remote_branch_ref = match git_config.get_remote_branch_ref(name_branch)
    {
        Some(rfs) => rfs,
        None => return Err(CommandsError::PullRemoteBranchNotFound),
    };
    println!("Remote branch ref: {}", remote_branch_ref);
    println!("Mergeando con el repositorio remoto ...");
    let merge_result = git_merge(repo_local, &remote_branch_ref)?;
    if merge_result.contains("CONFLICT") {
        let path_conflict = get_conflict_path(&merge_result);
        status.push(format!("Error: El siguiente archivo se sobrescribiría al fusionarlo:\n\t{}\nAborting.", path_conflict));
        status.push("No se puede hacer pull ya que hay conflictos".to_string());
        return Ok(status.join("\n"));
    }
    

    // Actualizo el fetch_head si todo salio bien
    fetch_head.branch_already_merged(current_rfs.get_name())?;
    fetch_head.write(repo_local)?;

    Ok("Pullcito naciendo".to_string())
}


// Err(_) => return Err(CommandsError::PullCurrentBranchNotFound),
