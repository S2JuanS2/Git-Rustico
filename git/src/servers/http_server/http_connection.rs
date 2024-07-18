use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use crate::errors::GitError;
use crate::util::logger::{get_client_signature, log_client_connect};
// use crate::servers::errors::ServerError;
use super::http_request::HttpRequest;


pub fn handle_client_http(
    stream: &mut TcpStream,
    tx: Arc<Mutex<Sender<String>>>,
    root_directory: String,
) -> Result<(), GitError> {
    // Loggear la conexión del cliente
    log_client_connect(stream, &tx);
    let _signature = get_client_signature(stream)?;
    // Creo la solicitud HTTP
    let request = HttpRequest::new_from_reader(stream)?;
    // Manejar la solicitud HTTP
    let _response = request.handle_http_request(&root_directory, &tx)?;
    // // Enviar la respuesta al cliente
    // send_response(stream, &response)?;
    
    Ok(())
}