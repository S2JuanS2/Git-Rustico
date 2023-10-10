use std::net::TcpListener;
use git::errors::GitError;

fn main()
{
    // Escucha en la dirección IP y el puerto deseados
    let listener = match start_server("127.0.0.1:9000")
    {
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

fn start_server(ip: &str) -> Result<TcpListener, GitError>
{
    match TcpListener::bind(ip) {
        Ok(listener) => Ok(listener),
        Err(_) => Err(GitError::ServerConnectionError),
    }
}