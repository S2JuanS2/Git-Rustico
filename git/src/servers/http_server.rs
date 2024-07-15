use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use crate::errors::GitError;


pub fn handle_client_http(
    _stream: &mut TcpStream,
    _tx: Arc<Mutex<Sender<String>>>,
    _root_directory: String,
) -> Result<(), GitError> {
    print!("HTTP");
    Ok(())
}