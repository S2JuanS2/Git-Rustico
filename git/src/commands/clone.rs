use std::{
    io::{Read, Write},
    net::TcpStream,
};

use crate::{
    errors::GitError,
    util::request::{create_git_request, RequestCommand},
};

pub fn git_clone(socket: &mut TcpStream) -> Result<(), GitError> {
    // Prepara la solicitud "git-upload-pack" para el servidor
    let message = create_git_request(
        RequestCommand::UploadPack,
        "sisop_2023a_ricaldi".to_string(),
        "127.0.0.2".to_string(),
        "9418".to_string(),
    );

    // let mut response = String::new();

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
    let mut buffer = [0; 1000]; // Un búfer para almacenar los datos recibidos.
    match socket.read(&mut buffer) {
        Ok(n) => {
            if n == 0 {
                // No se recibieron datos
                println!("No se recibieron datos del servidor");
            } else {
                // Convierte los bytes en un String válido
                let response_str = String::from_utf8_lossy(&buffer[0..n]);
                println!("Respuesta recibida del servidor: {}", response_str);
            }
        }
        Err(e) => {
            println!("Error al recibir la respuesta del servidor: {}", e);
            return Err(GitError::GenericError);
        }
    };

    // Procesa la respuesta, descarga y almacena los objetos en el sistema de archivos local, etc.

    Ok(())
}

// 0033
// git-upload
// -pack /pro
// ject.git\0h
// ost=myserv
// er.com\0
