use std::net::{TcpStream};
use git::errors::GitError;

fn main()
{
    
    let socket = match start_client("127.0.0.1:8080") {
        Ok(s) => s,
        Err(e) => {
            println!("{}", e.message());
            return;
        },
    };


    return;
}


fn start_client(ip: &str) -> Result<TcpStream, GitError>
{
    match TcpStream::connect(ip) {
        Ok(socket) => Ok(socket),
        Err(_) => Err(GitError::ClientConnectionError),
    }
}