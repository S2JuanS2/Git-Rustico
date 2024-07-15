use git::errors::GitError;
use git::servers::server::{create_listener, initialize_config, 
    process_request, receive_request, start_logging, start_server_thread, 
    wait_for_threads};
use git::util::logger::{
    get_client_signature, log_client_connect,
    log_client_disconnection_success,
};
use std::net::TcpStream;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};


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
fn handle_client_daemon(
    stream: &mut TcpStream,
    tx: Arc<Mutex<Sender<String>>>,
    root_directory: String,
) -> Result<(), GitError> {
    log_client_connect(stream, &tx);
    let signature = get_client_signature(stream)?;

    let request = receive_request(stream, signature.clone(), tx.clone())?;

    process_request(stream, &tx, &signature, &request, &root_directory)?;

    log_client_disconnection_success(&tx, &signature);
    Ok(())
}



fn handle_client_http(
    _stream: &mut TcpStream,
    _tx: Arc<Mutex<Sender<String>>>,
    _root_directory: String,
) -> Result<(), GitError> {
    print!("HTTP");
    Ok(())
}

/// Punto de entrada del servidor Git y servidor HTTP.
///
/// Esta función configura y lanza los servidores de Git y HTTP, y maneja la 
/// recepción y procesamiento de las conexiones de los clientes.
/// 
/// # Returns
///
/// Retorna un `Result` que contiene `()` en caso de éxito o un `GitError` en caso de fallo.
/// 
fn main() -> Result<(), GitError> {
    let config = initialize_config()?;
    print!("{}", config);

    let listener_daemon = create_listener(&config.ip, &config.port_daemon)?;
    let listener_http = create_listener(&config.ip, &config.port_http)?;

    let (shared_tx, log_handle) = start_logging(config.path_log)?;

    let clients_daemon_handle = start_server_thread(listener_daemon, Arc::clone(&shared_tx), config.src.clone(), handle_client_daemon)?;
    let clients_http_handle = start_server_thread(listener_http, shared_tx, config.src.clone(), handle_client_http)?;

    wait_for_threads(log_handle, clients_daemon_handle, clients_http_handle);

    Ok(())
}