use std::net::TcpStream;

use crate::errors::GitError;
use crate::util::advertised::AdvertisedRefs;
use crate::util::request::{create_git_request, RequestCommand};
use crate::util::connections::{send_message, end_connection};

use crate::util::pkt_line;

pub fn git_clone(socket: &mut TcpStream) -> Result<(), GitError> {
    // Prepara la solicitud "git-upload-pack" para el servidor
    let message = create_git_request(
        RequestCommand::UploadPack,
        "sisop_2023a_ricaldi".to_string(),
        "127.0.0.2".to_string(),
        "9418".to_string(),
    );

    // Envía la solicitud "git-upload-pack" al servidor
    send_message(socket, message)?;

    println!("Esperando respuesta del servidor...");

    let lines = pkt_line::read(socket)?;
    println!("Respuesta del servidor:");
    for line in &lines {
        println!("{}", String::from_utf8_lossy(&line));
    }
    
    let advertised = AdvertisedRefs::classify_vec(&lines)?;
    for a in advertised {
        println!("{}", a);
    }


    end_connection(socket)?;
    Ok(())
}