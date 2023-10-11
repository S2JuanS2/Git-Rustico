use crate::errors::GitError;
use std::net::TcpListener;
use std::net::TcpStream;

pub fn start_server(ip: &str) -> Result<TcpListener, GitError> {
    match TcpListener::bind(ip) {
        Ok(listener) => Ok(listener),
        Err(_) => Err(GitError::ServerConnectionError),
    }
}

pub fn start_client(ip: &str) -> Result<TcpStream, GitError> {
    match TcpStream::connect(ip) {
        Ok(socket) => Ok(socket),
        Err(_) => Err(GitError::ClientConnectionError),
    }
}
