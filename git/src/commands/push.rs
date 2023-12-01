use super::errors::CommandsError;
use crate::commands::config::GitConfig;
use crate::git_transport::git_request::GitRequest;
use crate::git_transport::references::{Reference, reference_discovery};
use crate::git_transport::request_command::RequestCommand;
use crate::models::client::Client;
use crate::util::connections::start_client;
use std::net::TcpStream;
use std::path::Path;

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
pub fn handle_push(args: Vec<&str>, client: Client) -> Result<String, CommandsError> {
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
// Quiero actualizar mi branch actual con los cambios del repositorio remoto
pub fn git_push(
    socket: &mut TcpStream,
    ip: &str,
    port: &str,
    repo_local: &str,
) -> Result<String, CommandsError> {
    // Obtengo el repositorio remoto
    println!("Push del repositorio remoto ...");
    let current_rfs = match Reference::get_current_references(repo_local)
    {
        Ok(rfs) => rfs,
        Err(_) => return Err(CommandsError::PullCurrentBranchNotFound),
    };
    let _git_config = GitConfig::new_from_file(repo_local)?;
    
    // Codear esto que lo necesita tambien el fetch
    // El remoto debe ser dinamico no estatico, y no debe haber remotos repetidos
    // let repo_remoto = git_config.get_remote_url(current_rfs.get_name());
    
    // Valido si la branch existe en el repositorio local
    let name_branch = current_rfs.get_name();
    let rfs_fetch = format!("refs/heads/{}", name_branch);
    let path_complete = format!("{}/.git/{}", repo_local, rfs_fetch);
    if !Path::new(&path_complete).exists() {
        return Ok("No hay cambios para hacer push".to_string());
    }

    // Prepara la solicitud "git-upload-pack" para el servidor
    let repo_remoto = "a codear...";
    let message =
        GitRequest::generate_request_string(RequestCommand::ReceivePack, repo_remoto, ip, port);
    
    let server = reference_discovery(socket, message, repo_remoto, &Vec::new())?;
    println!("Server: {:?}", server);


    Ok("Hola, soy baby push!".to_string())
}
