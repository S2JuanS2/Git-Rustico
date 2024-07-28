use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use crate::errors::GitError;
use super::server::{process_request, receive_request};



/// Maneja la conexión de un cliente, incluyendo la recepción y procesamiento de solicitudes.
///
/// # Arguments
///
/// * `stream` - Un mutable de referencia a la conexión TCP del cliente.
/// * `tx` - Un Arc de un Mutex que contiene el transmisor para enviar mensajes de registro.
/// * `root_directory` - Una cadena que representa el directorio raíz.
///
/// # Returns
///
/// Retorna un `Result` que contiene `()` en caso de éxito o un `GitError` en caso de fallo.
/// 
pub fn handle_client_daemon(
    stream: &mut TcpStream,
    signature: String,
    tx: &Arc<Mutex<Sender<String>>>,
    root_directory: String
) -> Result<(), GitError> {
    match _handle_client_daemon(stream, root_directory, tx, &signature) {
        Ok(_) => Ok(()),
        Err(e) => Err(GitError::RequestFailed(e.to_string())),
    }
}

pub fn _handle_client_daemon(
    stream: &mut TcpStream,
    root_directory: String,
    tx: &Arc<Mutex<Sender<String>>>,
    signature: &String,
) -> Result<(), GitError> 
{
    let request = receive_request(stream, signature.clone(), tx.clone())?;
    process_request(stream, &tx, &signature, &request, &root_directory)
}