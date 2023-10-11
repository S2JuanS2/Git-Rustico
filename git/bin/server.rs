use git::config::Config;
use git::util::connections::start_server;
// use git::errors::GitError;
// use std::net::TcpListener;
// use std::io::{Write, Read};

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

    match listener.accept() {
        Ok((_socket, address)) => println!("Nueva conexión: {}", address),
        Err(e) => println!("Error: {}", e),
    }
}

// fn mi_funcion() {
//     let listener = match start_server("127.0.0.1:27000")
//         {
//             Ok(listener) => listener,
//             Err(e) => {
//                 println!("{}", e.message());
//                 return;
//             }
//         };
//         println!("Servidor escuchando en 127.0.0.1:8080");
//         println!("Servidor escuchando en 127.0.0.1:27000");

//         match listener.accept() {
//             Ok((_socket, address)) => println!("Nueva conexión: {}", address),
//             Ok((mut socket, address)) => {
//                 println!("Nueva conexión: {}", address);

//                 let mut command = String::new();
//                 match socket.read_to_string(&mut command) {
//                     Ok(_) => println!("Comando recibido: {}", command),
//                     Err(e) => {
//                         println!("Error: {}", e);
//                         return;
//                     }
//                 }
//                 println!("Lei algo del client");

//                 // Procesar el comando y simular una respuesta de "status"
//                 let mut status_response = "On branch main\0";
//                 // status_response.push('\0');
//                 // match socket.write(status_response.as_bytes()) {
//                 match socket.write(b"lala\0") {
//                     Ok(_) => {println!("Envie la respuesta al cliente")},
//                     Err(e) => {
//                         println!("Error: {}", e);
//                         return;
//                     }
//                 }
//             },
//             Err(e) => println!("Error: {}", e),
//         }
//         return;
// }
