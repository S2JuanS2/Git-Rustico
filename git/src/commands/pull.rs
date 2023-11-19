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
    if args.len() >= 1 {
        return Err(CommandsError::InvalidArgumentCountPull.into());
    }
    let mut socket = start_client(client.get_address())?;
    git_pull(&mut socket, client.get_ip(), client.get_port(), client.get_directory_path())
}

/// actualiza el repositorio local con los cambios del repositorio remoto
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'socket': socket del cliente
/// 'ip': ip del cliente
/// 'port': puerto del cliente
pub fn git_pull(
    _socket: &mut TcpStream,
    _ip: &str,
    _port: &str,
    _repo: &str,
) -> Result<(), GitError> {
    Ok(())
}
