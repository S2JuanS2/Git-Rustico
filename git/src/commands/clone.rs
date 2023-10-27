use std::net::TcpStream;

use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::connections::{
    packfile_negotiation, receive_packfile, reference_discovery, start_client,
};
use crate::util::request::{create_git_request, RequestCommand};

/// Esta función se encarga de llamar a al comando clone con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función clone
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_clone(args: Vec<&str>, client: Client) -> Result<(), GitError> {
    let address = client.get_ip();
    if args.len() != 1 {
        return Err(GitError::CloneMissingRepoError);
    }
    let mut socket = start_client(&address)?;
    let parts = address.split(':').collect::<Vec<&str>>();
    let ip = parts[0].to_string();
    let port = parts[1].to_string();
    git_clone(&mut socket, ip, port, args[0].to_string())
}

/// Esta función se encarga de clonar un repositorio remoto
/// ###Parametros:
/// 'socket': Socket que se utiliza para comunicarse con el servidor
/// 'ip': Dirección ip del servidor
/// 'port': Puerto del servidor
/// 'repo': Nombre del repositorio que se quiere clonar
pub fn git_clone(
    socket: &mut TcpStream,
    ip: String,
    port: String,
    repo: String,
) -> Result<(), GitError> {
    // Prepara la solicitud "git-upload-pack" para el servidor
    let message = create_git_request(RequestCommand::UploadPack, repo, ip, port);

    // Reference Discovery
    let advertised = reference_discovery(socket, message)?;

    // Packfile Negotiation
    packfile_negotiation(socket, advertised)?;

    // Packfile Data
    receive_packfile(socket)?;

    Ok(())
}
