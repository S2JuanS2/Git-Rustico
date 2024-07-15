use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
// use http_request::HttpRequest;

use crate::errors::GitError;
// use super::errors::ServerError;
pub mod http_request;
pub mod utils;


pub fn handle_client_http(
    stream: &mut TcpStream,
    _tx: Arc<Mutex<Sender<String>>>,
    _root_directory: String,
) -> Result<(), GitError> {
    print!("HTTP");
    // Leer datos del cliente
    let _request = utils::read_request(stream)?;
    
    // // Parsear la solicitud HTTP
    // let request = parse_http_request(&request_data)?;

    // print!("{:?}", request);
    // // Manejar la solicitud HTTP
    // let response = handle_http_request(request, tx)?;
    
    // // Enviar la respuesta al cliente
    // send_response(stream, &response)?;
    
    Ok(())
}

