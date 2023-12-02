use super::errors::CommandsError;
use crate::commands::config::GitConfig;
use crate::git_transport::git_request::GitRequest;
use crate::git_transport::references::{Reference, reference_discovery};
use crate::git_transport::request_command::RequestCommand;
use crate::models::client::Client;
use crate::util::connections::start_client;
use std::net::TcpStream;


struct PushBranch{
    path_local: String,
    name_remoto: String,
    url_remote: String,
    git_config: String,
    name_branch: String,
    status: Vec<String>
}


/// Comandos que aceptare:
/// git push -> push de la rama actual
/// git push all -> push de todas las ramas
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
    if args.len() > 1 {
        return Err(CommandsError::InvalidArgumentCountPush);
    }
    
    let repo_local = client.get_directory_path();
    let mut socket = start_client(client.get_address())?;
    
    if args.is_empty() {
        let branch = match Reference::get_current_references(&repo_local)
        {
            Ok(rfs) => rfs,
            Err(_) => return Err(CommandsError::PushCurrentBranchNotFound),
        
        };
        return git_push_branch (
            &mut socket,
            client.get_ip(),
            client.get_port(),
            repo_local,
            &branch,
        );
    }
    if args[1] != "all" {
        return Err(CommandsError::InvalidArgumentPush);
    }
    git_push_all(
        &mut socket,
        client.get_ip(),
        client.get_port(),
        repo_local,
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
pub fn git_push_branch(
    socket: &mut TcpStream,
    ip: &str,
    port: &str,
    repo_local: &str,
    branch: &Reference,
) -> Result<String, CommandsError> {
    // Obtengo el repositorio remoto
    let git_config = GitConfig::new_from_file(repo_local)?;
    let remote_name = git_config.get_remote_by_branch_name(branch.get_name())?;
    let url_remote = git_config.get_remote_url_by_name(&remote_name)?;

    let mut status: Vec<String> = Vec::new();
    status.push(format!("Pushing to {}...", branch.get_name()));
    status.push(format!("\tRepositorio local {}", repo_local));
    status.push(format!("\tRepositorio remoto {}", remote_name));
    status.push(format!("\tUrl del remoto: {}", url_remote));

    // Prepara la solicitud "git-upload-pack" para el servidor
    let message =
        GitRequest::generate_request_string(RequestCommand::ReceivePack, &url_remote, ip, port);
    
    let server = reference_discovery(socket, message, &url_remote, &Vec::new())?;
    if !server.contains_reference(&branch.get_ref_path())
    {
        // Se debe crear la branch
        // 00000000000000000 hashactual
    }


    Ok("Hola, soy baby push!".to_string())
}


fn git_push_all(
    _socket: &mut TcpStream,
    _ip: &str,
    _port: &str,
    _repo_local: &str,
) -> Result<String, CommandsError> {
    Ok("Hola, soy baby push all!".to_string())
}