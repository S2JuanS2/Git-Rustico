use std::sync::Arc;
use git::errors::GitError;
use git::servers::daemon_server::handle_client_daemon;
use git::servers::http_server::connection::handle_client_http;
use git::servers::server::{create_listener, initialize_config, 
    start_logging, start_server_thread, 
    wait_for_threads};

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