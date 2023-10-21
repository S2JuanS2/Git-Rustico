use std::io::Read;
use std::net::TcpStream;

use crate::errors::GitError;
use crate::util::advertised::AdvertisedRefs;
use crate::util::pkt_line::add_length_prefix;
use crate::util::request::{create_git_request, RequestCommand};
use crate::util::connections::{end_connection, reference_discovery, send_message};

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
    for a in advertised {
        let message = match a {
            AdvertisedRefs::Ref { obj_id, ref_name: _ } => format!("want {}\n", obj_id),
            // AdvertisedRefs::Shallow { obj_id } => format!("want {}\n", obj_id),
            _ => continue,
        };
        let message = add_length_prefix(&message, message.len());
        print!("{}", message);
        send_message(socket, message)?;
    }
    end_connection(socket)?;
    let message = "0009done\n";
    send_message(socket, message.to_string())?;

    let mut buffer = [0; 4096];  // Tamaño del búfer de lectura
    match socket.read(&mut buffer)
    {
        Ok(_) => {
            let m = String::from_utf8(buffer.to_vec()).expect("No se pudo convertir a String");
            println!("Lectura exitosa: {:?}", m);
        }
        Err(e) => 
        {
            println!("Error: {}", e);
            return Err(GitError::GenericError);
        }
    };

    let mut buffer = [0; 4096];  // Tamaño del búfer de lectura
    match socket.read(&mut buffer)
    {
        Ok(_) => {
            let m = String::from_utf8(buffer.to_vec()).expect("No se pudo convertir a String");
            println!("Lectura exitosa: {:?}", m);
        }
        Err(e) => 
        {
            println!("Error: {}", e);
            return Err(GitError::GenericError);
        }
    };

    Ok(())
}