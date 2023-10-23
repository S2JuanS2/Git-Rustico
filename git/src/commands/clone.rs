use std::net::TcpStream;

use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::connections::{packfile_negotiation, receive_packfile, reference_discovery, start_client};
use crate::util::request::{create_git_request, RequestCommand};

/// Esta función se encarga de llamar a al comando clone con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función clone
pub fn handle_clone(args: Vec<&str>, client: Client) -> Result<(), GitError> {
    if args.len() != 1 {
        return Err(GitError::InvalidArgumentCountCloneError);
    }
    let mut socket = start_client(&client.get_ip())?;
    git_clone(&mut socket)
}

/// Esta función se encarga de clonar un repositorio remoto
/// ###Parametros:
/// 'socket': Socket que se utiliza para comunicarse con el servidor
pub fn git_clone(socket: &mut TcpStream) -> Result<(), GitError> {
    // Prepara la solicitud "git-upload-pack" para el servidor
    let message = create_git_request(
        RequestCommand::UploadPack,
        "sisop_2023a_ricaldi".to_string(),
        "127.0.0.2".to_string(),
        "9418".to_string(),
    );

    // Reference Discovery
    let advertised = reference_discovery(socket, message)?;

    // Packfile Negotiation
    packfile_negotiation(socket, advertised)?;

    // Packfile Data
    receive_packfile(socket)?;

    Ok(())
}
