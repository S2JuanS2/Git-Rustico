use std::net::TcpStream;

use crate::errors::GitError;
use crate::util::connections::{packfile_negotiation, reference_discovery, start_client, receive_packfile};
use crate::util::request::{create_git_request, RequestCommand};

pub fn handle_clone(address: &str) -> Result<(), GitError> {
    let mut socket = start_client(address)?;
    git_clone(&mut socket)?;
    Ok(())
}

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
