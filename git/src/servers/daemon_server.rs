use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use crate::consts::DAEMON_SIGNATURE;
use crate::errors::GitError;
use crate::util::logger::{get_client_signature, log_client_connect, log_client_disconnection_success};
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
    tx: Arc<Mutex<Sender<String>>>,
    root_directory: String,
) -> Result<(), GitError> {
    log_client_connect(stream, &tx, &DAEMON_SIGNATURE.to_string());
    let signature = get_client_signature(stream, &DAEMON_SIGNATURE.to_string())?;

    let request = receive_request(stream, signature.clone(), tx.clone())?;

    process_request(stream, &tx, &signature, &request, &root_directory)?;

    log_client_disconnection_success(&tx, &signature);
    Ok(())
}