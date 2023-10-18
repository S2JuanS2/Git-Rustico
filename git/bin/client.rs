use git::config::Config;
use git::util::connections::start_client;
use std::{io, env};
use std::io::Write;

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

    let addres = format!("{}:{}", config.ip, config.port);
    println!("Me voy a conectar a: {}", addres);

    match start_client(&addres) {
        Ok(mut stream) => {
            println!("Ingrese un comando Git: ");

            let mut input = String::new();

            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let command_bytes = input.trim().as_bytes();
                    if let Err(error) = stream.write(command_bytes) {
                        eprintln!("Error al escribir: {}", error);
                    }
                }
                Err(error) => {
                    eprintln!("Error al leer la entrada: {}", error);
                }
            }
        }
        Err(e) => {
            println!("{}", e.message());
        }
    };
}
