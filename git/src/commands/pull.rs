use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::connections::start_client;
use std::net::TcpStream;

/// Esta función se encarga de llamar al comando push con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función push
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_pull(args: Vec<&str>, client: Client) -> Result<(), GitError> {
    let client_clone = client.clone();
    let address = client_clone.get_address();
    if args.is_empty() {
        return Err(GitError::InvalidArgumentCountPullError);
    }
    let directory = client.get_directory_path();
    let mut socket = start_client(address)?;
    let parts = address.split(':').collect::<Vec<&str>>();
    let ip = parts[0].to_string();
    let port = parts[1].to_string();
    git_pull(directory, &mut socket, ip, port)
}

/// actualiza el repositorio local con los cambios del repositorio remoto
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'socket': socket del cliente
/// 'ip': ip del cliente
/// 'port': puerto del cliente
pub fn git_pull(
    _directory: &str,
    _socket: &mut TcpStream,
    _ip: String,
    _port: String,
) -> Result<(), GitError> {
    Ok(())
}
