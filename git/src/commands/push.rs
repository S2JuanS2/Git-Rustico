use super::errors::CommandsError;
use crate::models::client::Client;
use crate::util::connections::start_client;
use std::net::TcpStream;

/// Maneja el comando "push" en el servidor Git.
///
/// # Arguments
///
/// * `args` - Argumentos proporcionados al comando. Se espera que esté vacío ya que "push" no requiere argumentos.
/// * `client` - Objeto `Client` que contiene la información del cliente, como la dirección, el puerto y la ruta del directorio.
///
/// # Returns
///
/// Retorna un resultado que indica si la operación "push" fue exitosa o si hubo errores durante la ejecución.
///
/// # Errors
///
/// Retorna un error si la cantidad de argumentos no es la esperada o si hay problemas al iniciar la conexión con el cliente o ejecutar el comando "git push".
///
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
