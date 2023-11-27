use crate::commands::fetch::git_fetch_all;
use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::connections::start_client;
use std::net::TcpStream;

use super::errors::CommandsError;

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
/// * `Result<(), GitError>` - Un resultado que indica el éxito o un error encontrado durante la operación de pull.
///
/// # Errores
///
/// * `CommandsError::InvalidArgumentCountPull` - Indica que se proporcionó un número incorrecto de argumentos para el comando pull.
/// * `GitError` - Indica varios errores relacionados con Git que podrían ocurrir durante la operación de pull.
///
pub fn handle_pull(args: Vec<&str>, client: Client) -> Result<(), GitError> {
    if !args.is_empty() {
        return Err(CommandsError::InvalidArgumentCountPull.into());
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
) -> Result<(), GitError> {
    println!("Pull del repositorio remoto ...");

    match git_fetch_all(socket, ip, port, repo_local)
    {
        Ok(_) => print!("Se descargo las actualizaciones del repositorio remoto con exito"),
        Err(e) => return Err(e.into()),
    }

    // TODO: Implementar el merge de los cambios
        


    Ok(())
}
