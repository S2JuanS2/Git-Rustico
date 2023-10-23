use std::io::Read;
use std::net::TcpStream;

use crate::errors::GitError;
use crate::util::connections::{packfile_negotiation, reference_discovery};
use crate::util::request::{create_git_request, RequestCommand};


/// Esta función se encarga de llamar a al comando clone con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene el comando que se le pasara al servidor
pub fn handle_clone(args: Vec<&str>) -> Result<(), GitError> {
    if args.len() != 1 {
        return Err(GitError::InvalidArgumentCountCloneError);
    }
    Ok(())
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

    let mut buffer = [0; 4096]; // Tamaño del búfer de lectura
    match socket.read(&mut buffer) {
        Ok(_) => {
            let m = String::from_utf8(buffer.to_vec()).expect("No se pudo convertir a String");
            println!("Lectura exitosa: {:?}", m);
        }
        Err(e) => {
            println!("Error: {}", e);
            return Err(GitError::GenericError);
        }
    };

    let mut buffer = [0; 4096]; // Tamaño del búfer de lectura
    match socket.read(&mut buffer) {
        Ok(_) => {
            let m = String::from_utf8(buffer.to_vec()).expect("No se pudo convertir a String");
            println!("Lectura exitosa: {:?}", m);
        }
        Err(e) => {
            println!("Error: {}", e);
            return Err(GitError::GenericError);
        }
    };

    Ok(())
}
