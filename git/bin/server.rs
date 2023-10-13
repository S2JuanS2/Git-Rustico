use std::io::Read;

use git::config::Config;
use git::util::connections::start_server;

fn main() {
    let config = match Config::new() {
        Ok(config) => config,
        Err(error) => {
            println!("Error: {}", error.message());
            return;
        }
    };
    print!("{}", config);

    let address = format!("{}:{}", config.ip, config.port);
    println!("{}", address);

    // Escucha en la dirección IP y el puerto deseados
    let listener = match start_server(&address) {
        Ok(listener) => listener,
        Err(e) => {
            println!("{}", e.message());
            return;
        }
    };
    println!("Servidor escuchando en 127.0.0.1:8080");

    let (mut stream, _address) = match listener.accept() {
        Ok((stream, address)) => {
            println!("Nueva conexión: {}", address);
            (stream, address)
        }
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };
    let mut buffer = [0; 1024]; // Un búfer para almacenar los datos recibidos.

    match stream.read(&mut buffer) {
        Ok(n) => {
            if n > 0 {
                // Convierte los bytes leídos en una cadena y muestra el mensaje.
                let message = String::from_utf8_lossy(&buffer[..n]);
                println!("Mensaje del cliente: {}", message);
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
