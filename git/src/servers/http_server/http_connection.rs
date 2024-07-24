use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use crate::consts::HTPP_SIGNATURE;
use crate::errors::GitError;
use crate::util::logger::{get_client_signature, log_client_connect, log_client_disconnection_error, log_client_disconnection_success, log_message_with_signature, log_request_error};
use super::http_request::HttpRequest;
use super::status_code::StatusCode;
use super::utils::send_response_http;


pub fn handle_client_http(
    stream: &mut TcpStream,
    tx: Arc<Mutex<Sender<String>>>,
    root_directory: String,
) -> Result<(), GitError> {
    // Loggear la conexión del cliente
    log_client_connect(stream, &tx, &HTPP_SIGNATURE.to_string());
    let signature = get_client_signature(stream, &HTPP_SIGNATURE.to_string())?;

    match _handle_client_http(stream, root_directory, &tx, &signature) {
        Ok(status_code) => {
            let message = format!("Response sent to client with status code: {}", status_code);
            log_message_with_signature(&tx, &HTPP_SIGNATURE.to_string(), &message);
            send_response_http(stream, status_code)?;
            log_client_disconnection_success(&tx, &signature);
            Ok(())
        },
        Err(e) => {
            let status_code = StatusCode::InternalError;
            send_response_http(stream, status_code)?;
            log_request_error(&e.to_string(), &signature, &tx);
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
) -> Result<StatusCode, GitError> {
    // Creo la solicitud HTTP
    let request = HttpRequest::new_from_reader(stream)?;
    // Manejar la solicitud HTTP
    Ok(request.handle_http_request(&root_directory, tx, &signature)?)
}

