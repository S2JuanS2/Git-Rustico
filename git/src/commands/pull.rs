use crate::commands::config::GitConfig;
use crate::commands::fetch::git_fetch_branch;
use crate::commands::fetch_head::FetchHead;
use crate::commands::merge::{git_merge, get_conflict_path};
use crate::git_transport::references::Reference;
use super::errors::CommandsError;
use crate::models::client::Client;
use crate::util::connections::start_client;
use std::net::TcpStream;


/// Acepto:
/// git pull -> pull del branch actual
/// git pull <branch> <remote> -> pull del branch especificado del repositorio remoto especificado
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
    if !args.is_empty() && args.len() != 2 {
        return Err(CommandsError::InvalidArgumentCountPull);
    }

    let mut status = Vec::new();
    let path_repo = client.get_directory_path(); 
    if args.len() == 2 
    {
        let name_branch = args[1];
        let name_remote = args[2];
        status.push(format!("Branch local: {}", args[0]));
        status.push(format!("Remoto: {}", args[1]));
        let current_rfs = Reference::get_current_references(path_repo)?;
        let mut git_config: GitConfig = GitConfig::new_from_file(path_repo)?;
        if !git_config.valid_remote(name_remote)
        {
            status.push(format!("El repositorio remoto {} no existe", name_remote));
            return Ok(status.join("\n"));
        };
        git_config.add_branch(current_rfs.get_name(), name_remote, &format!("refs/heads/{}", name_branch))?;
        git_config.write_to_file(path_repo)?;
        status.push("Se asocio el branch local con el remoto".to_string());

    }
    let mut socket = start_client(client.get_address())?;

    git_pull(
        &mut socket,
        client.get_ip(),
        client.get_port(),
        client.get_directory_path(),
        client.clone(),
        &mut status,
    )
}

// Quiero actualizar mi branch actual con los cambios del repositorio remoto
pub fn git_pull(
    socket: &mut TcpStream,
    ip: &str,
    port: &str,
    repo_local: &str,
    client: Client,
    status: &mut Vec<String>,
) -> Result<String, CommandsError> {
    // Obtengo el repositorio remoto
    println!("Pull del repositorio remoto ...");
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
    let merge_result = git_merge(repo_local, &remote_branch_ref, client)?;
    println!("Result del merge: {}", merge_result);
    if merge_result.contains("CONFLICT") {
        let path_conflict = get_conflict_path(&merge_result);
        status.push(format!("Error: El siguiente archivo se sobrescribiría al fusionarlo:\n\t{}\nAborting.", path_conflict));
        status.push("No se puede hacer pull ya que hay conflictos".to_string());
        return Ok(status.join("\n"));
    }
    

    // Actualizo el fetch_head si todo salio bien
    fetch_head.branch_already_merged(current_rfs.get_name())?;
    fetch_head.write(repo_local)?;

    status.push("Baby pull :)".to_string());
    Ok(status.join("\n"))
}


// Err(_) => return Err(CommandsError::PullCurrentBranchNotFound),
