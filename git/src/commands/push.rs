use super::errors::CommandsError;
use crate::models::client::Client;
use crate::util::connections::start_client;
use std::net::TcpStream;

/// Esta función se encarga de llamar al comando push con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función push
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_push(args: Vec<&str>, client: Client) -> Result<(), CommandsError> {
    if !args.is_empty() {
        return Err(CommandsError::InvalidArgumentCountPush);
    }
    let mut socket = start_client(client.get_address())?;
    git_push(
        &mut socket,
        client.get_ip(),
        client.get_port(),
        client.get_directory_path(),
    )
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
    socket: &mut TcpStream,
    ip: &str,
    port: &str,
    repo_local: &str,
) -> Result<(), CommandsError> {
    Ok(())
}
