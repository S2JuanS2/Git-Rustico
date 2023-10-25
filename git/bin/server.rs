use git::config::Config;
use git::errors::GitError;
use git::util::connections::start_server;
use std::env;
use std::io::Read;
use std::net::TcpStream;

fn handle_client(mut stream: TcpStream) {
    // Leer datos del cliente
    let mut buffer = [0; 1024];

    if let Err(error) = stream.read(&mut buffer) {
        eprintln!("Error al leer: {}", error);
        return;
    }
    println!("Recibido: {}", String::from_utf8_lossy(&buffer));
}

fn main() -> Result<(), GitError> {
    let args: Vec<String> = env::args().collect();
    let config =  Config::new(args)?;
    print!("{}", config);

    let address = format!("{}:{}", config.ip, config.port);

    // Escucha en la dirección IP y el puerto deseados
    let listener = start_server(&address)?;
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

    Ok(())
}
