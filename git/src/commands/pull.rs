use crate::commands::branch::get_current_branch;
use crate::commands::fetch::{git_fetch_branch, FetchStatus};
use crate::commands::fetch_head::{self, FetchHead};
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

pub fn git_pull(
    socket: &mut TcpStream,
    ip: &str,
    port: &str,
    repo_local: &str,
) -> Result<String, CommandsError> {
    // Obtengo el repositorio remoto
    println!("Pull del repositorio remoto ...");

    let name_branch = match get_current_branch(repo_local)
    {
        Ok(name_branch) => name_branch,
        Err(_) => return Err(CommandsError::PullCurrentBranchNotFound), // Pensar porque fallaria esto?
    };
    let result =  git_fetch_branch(socket, ip, port, repo_local, &name_branch)?;
    match result {
        FetchStatus::BranchNotFound => return Ok(format!("{}", FetchStatus::BranchNotFound)),
        FetchStatus::NoUpdates => return Ok(format!("{}", FetchStatus::NoUpdates)),
        FetchStatus::Success => (),
    }

    // let fetch_head = FetchHead::new_from_file(repo_local)?;



    Ok("Pullcito naciendo".to_string())
}
