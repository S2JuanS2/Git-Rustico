use git::config::Config;
use git::util::connections::start_server;
use std::env;
use std::io::Read;
use std::net::TcpStream;
use crate::commands::branch::handle_branch;
use crate::commands::clone::handle_clone;
use crate::commands::commit::handle_commit;
use crate::commands::init::handle_init;
use crate::commands::cat_file::handle_cat_file;
use crate::commands::add::handle_add;
use crate::commands::status::handle_status;
use crate::commands::checkout::handle_checkout;
use crate::commands::fetch::handle_fetch;
use crate::commands::hash_object::handle_hash_object;
use crate::errors::GitError;


/// Esta función se encarga de llamar a al comando adecuado con los parametros necesarios
/// ###Parametros:
/// 'buffer': String que contiene el comando que se le pasara al servidor
fn handle_command(buffer: String)-> Result<(), GitError> {
    let command = buffer.trim();
    let commands = command.split_whitespace().collect::<Vec<&str>>();
    let rest_of_command = commands.iter().skip(2).cloned().collect::<Vec<&str>>();
    if commands[0] == "git" {
        match commands[1] {
            "branch" => {
                handle_branch(rest_of_command);
            }
            "clone" => {
                handle_clone(rest_of_command);
            }
            "commit" => {
                handle_commit(rest_of_command);
            }
            "init" => {
                handle_init(rest_of_command);
            }
            "cat_file" => {
                handle_cat_file(rest_of_command);
            }
            "add" => {
                handle_add(rest_of_command);
            }
            "status" => {
                handle_status(rest_of_command);
            }
            "checkout" => {
                handle_checkout(rest_of_command);
            }
            "fetch" => {
                handle_fetch(rest_of_command);
            }
            "hash_object" => {
                handle_hash_object(rest_of_command);
            }
            _ => {
                return Err(GitError::CommandNotRecognizedError);
            }
        }
    } else {
        return Err(GitError::CommandDoesNotStartWithGitError);
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream) {
    // Leer datos del cliente
    let mut buffer = [0; 1024];

    if let Err(error) = stream.read(&mut buffer) {
        eprintln!("Error al leer: {}", error);
        return;
    }
    println!("Recibido: {}", String::from_utf8_lossy(&buffer));

    handle_command(String::from_utf8_lossy(&buffer));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = match Config::new(args) {
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
    println!("Servidor escuchando en {}", address);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Nueva conexión: {:?}", stream.local_addr());
                std::thread::spawn(|| {
                    handle_client(stream);
                });
            }
            Err(e) => {
                eprintln!("Error al aceptar la conexión: {}", e);
            }
        }
    }
}
