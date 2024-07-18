use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use crate::consts::HTPP_SIGNATURE;
use crate::errors::GitError;
use crate::util::logger::{get_client_signature, log_client_connect, log_client_disconnection_error, log_client_disconnection_success};
// use crate::servers::errors::ServerError;
use super::http_request::HttpRequest;


pub fn handle_client_http(
    stream: &mut TcpStream,
    tx: Arc<Mutex<Sender<String>>>,
    root_directory: String,
) -> Result<(), GitError> {
        // Loggear la conexión del cliente
        log_client_connect(stream, &tx);
        let signature = get_client_signature(stream, &HTPP_SIGNATURE.to_string())?;

    match _handle_client_http(stream, root_directory, &tx, &signature) {
        Ok(_) => {
            log_client_disconnection_success(&tx, &signature);
            Ok(())
        },
        Err(e) => {
            log_client_disconnection_error(&tx, &signature);
            Err(e)
        }
    }
}


pub fn _handle_client_http(
    stream: &mut TcpStream,
    root_directory: String,
    tx: &Arc<Mutex<Sender<String>>>,
    signature: &String,
) -> Result<(), GitError> {
    // Creo la solicitud HTTP
    let request = HttpRequest::new_from_reader(stream)?;
    // Manejar la solicitud HTTP
    let _response = request.handle_http_request(&root_directory, tx, &signature)?;
    // // Enviar la respuesta al cliente
    // send_response(stream, &response)?;
    Ok(())
}

