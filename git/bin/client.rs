use git::errors::GitError;
use std::net::TcpStream;

use std::thread;
use std::time::Duration;

fn main()
{
    
    let _ = match start_client("127.0.0.1:9418") {
        Ok(s) => s,
        Err(e) => {
            println!("{}", e.message());
            return;
        },
    };
    println!("Estoy conectado con daemon");

    println!("Me voy a mimir 5 segundos");
    // Crea una duración de 5 segundos
    let cinco_segundos = Duration::from_secs(5);

    // Hace que el programa duerma durante 5 segundos
    thread::sleep(cinco_segundos);
    println!("Sali de mimir, me toca morir");
}


fn start_client(ip: &str) -> Result<TcpStream, GitError>
{
    match TcpStream::connect(ip) {
        Ok(socket) => Ok(socket),
        Err(_) => Err(GitError::ClientConnectionError),
    }
}