use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use crate::consts::APPLICATION_SERVER;
use crate::errors::GitError;
use crate::util::logger::log_message_with_signature;
use super::http_request::HttpRequest;
use super::status_code::StatusCode;
use super::utils::send_response_http;


/// Maneja las conexiones HTTP de los clientes.
///
/// Esta función se encarga de manejar una conexión HTTP entrante de un cliente, procesar la solicitud,
/// y enviar la respuesta adecuada. También se encarga de loggear los eventos de conexión, desconexión,
/// y posibles errores que ocurran durante el proceso.
///
/// # Argumentos
///
/// * `stream` - Un mutable referencia a un `TcpStream` que representa la conexión con el cliente.
/// * `tx` - Un `Arc<Mutex<Sender<String>>>` que se utiliza para enviar mensajes de log.
/// * `root_directory` - Un `String` que representa el directorio raíz del servidor.
///
/// # Retornos
///
/// Retorna un `Result<(), GitError>` indicando si la operación fue exitosa o si ocurrió un error.
///
pub fn handle_client_http(
    stream: &mut TcpStream,
    signature: String,
    tx: &Arc<Mutex<Sender<String>>>,
    root_directory: String
) -> Result<(), GitError> {
    let (request, status_code) = _handle_client_http(stream, root_directory, tx, &signature);
    let content_type = match request {
        Some(request) => request.get_content_type(),
        None => APPLICATION_SERVER.to_string(),
    };
    
    let message = format!("Response sent to client with status code: {}", status_code.to_string());
    log_message_with_signature(tx, &signature, &message);

    send_response_http(stream, &status_code, &content_type)?;

    match status_code {
        StatusCode::Ok(_) => Ok(()),
        _ => Err(GitError::RequestFailed(status_code.to_string()))
    }
}

/// Maneja las solicitudes HTTP internas del cliente.
///
/// Esta función auxiliar se encarga de crear la solicitud HTTP a partir del flujo de entrada,
/// y manejar la solicitud procesándola y retornando el código de estado apropiado.
///
/// # Argumentos
///
/// * `stream` - Un mutable referencia a un `TcpStream` que representa la conexión con el cliente.
/// * `root_directory` - Un `String` que representa el directorio raíz del servidor.
/// * `tx` - Una referencia a un `Arc<Mutex<Sender<String>>>` que se utiliza para enviar mensajes de log.
/// * `signature` - Una referencia a un `String` que contiene la firma del cliente.
///
/// # Retornos
///
/// Retorna un `Result<StatusCode, GitError>` indicando si la operación fue exitosa o si ocurrió un error.
///
pub fn _handle_client_http(
    stream: &mut TcpStream,
    root_directory: String,
    tx: &Arc<Mutex<Sender<String>>>,
    signature: &String,
) -> (Option<HttpRequest>, StatusCode) {
    // Creo la solicitud HTTP
    let request = match HttpRequest::new_from_reader(stream) {
        Ok(request) => request,
        Err(e) => return (None, e),
    };
    // Manejar la solicitud HTTP
    match request.handle_http_request(&root_directory, tx, signature)
    {
        Ok(status_code) => (Some(request), status_code),
        Err(e) => (Some(request), StatusCode::InternalError(e.to_string())),
    }
}


