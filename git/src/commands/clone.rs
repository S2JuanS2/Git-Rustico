use std::{
    io::{Read, Write},
    net::TcpStream,
};

use crate::{errors::GitError, util::request::{RequestCommand, create_git_request}};

pub fn git_clone(socket: &mut TcpStream) -> Result<(), GitError> {
    // Prepara la solicitud "git-upload-pack" para el servidor
    let message = create_git_request(
        RequestCommand::UploadPack,
        "sisop_2023a_ricaldi".to_string(),
        "127.0.0.2".to_string(),
        "9418".to_string(),
    );

    let mut response = String::new();

    // Envía la solicitud "git-upload-pack" al servidor
    match socket.write(message.as_bytes()) {
        Ok(_) => println!("Solicitud enviada al servidor"),
        Err(e) => {
            println!("Error al enviar la solicitud al servidor: {}", e);
            return Err(GitError::GenericError);
        }
    };

    println!("Esperando respuesta del servidor...");
    // Lee y procesa la respuesta del servidor
    match socket.read_to_string(&mut response) {
        Ok(_) => println!("Respuesta recibida del servidor"),
        Err(e) => {
            println!("Error al recibir la respuesta del servidor: {}", e);
            return Err(GitError::GenericError);
        }
    };

    // Procesa la respuesta, descarga y almacena los objetos en el sistema de archivos local, etc.
    println!("Respuesta del servidor: {}", response);

    Ok(())
}


// 0033
// git-upload
// -pack /pro
// ject.git\0h
// ost=myserv
// er.com\0
