use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::connections::start_client;
use std::net::TcpStream;

/// Esta función se encarga de llamar al comando push con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función push
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_push(args: Vec<&str>, client: Client) -> Result<(), GitError> {
    let client_clone = client.clone();
    let address = client_clone.get_address();
    if args.len() != 2 {
        return Err(GitError::InvalidArgumentCountPushError);
    }
    let directory = client.get_directory_path();
    let mut socket = start_client(address)?;
    let parts = address.split(':').collect::<Vec<&str>>();
    let ip = parts[0].to_string();
    let port = parts[1].to_string();
    let remote_name = args[0];
    let branch_name = args[1];
    git_push(directory, &mut socket, ip, port, remote_name, branch_name)
}

/// actualiza el repositorio remoto con los cambios del repositorio local
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'socket': socket del cliente
/// 'ip': ip del cliente
/// 'port': puerto del cliente
/// 'remote_name': nombre del repositorio remoto
/// 'branch_name': nombre de la rama a mergear
pub fn git_push(
    _directory: &str,
    _socket: &mut TcpStream,
    _ip: String,
    _port: String,
    _remote_name: &str,
    _branch_name: &str,
) -> Result<(), GitError> {
    Ok(())
}
