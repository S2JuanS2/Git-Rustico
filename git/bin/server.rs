use git::config::Config;
use git::errors::GitError;
use git::git_transport::git_request::GitRequest;
use git::util::connections::start_server;
use git::util::errors::UtilError;
use git::util::logger::{
    get_client_signature, handle_log_file, log_client_connect, log_client_disconnection_error,
    log_client_disconnection_success, log_message,
};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::{env, thread};

/// Recibe una solicitud del cliente y la procesa.
///
/// # Arguments
///
/// * `stream` - Un mutable de referencia a la conexión TCP del cliente.
/// * `signature` - Una cadena que representa la firma del cliente.
/// * `tx` - Un Arc de un Mutex que contiene el transmisor para enviar mensajes de registro.
///
/// # Returns
///
/// Retorna un `Result` que contiene una `GitRequest` en caso de éxito o un `GitError` en caso 
/// de fallo.
/// 
fn receive_request(
    stream: &mut TcpStream,
    signature: String,
    tx: Arc<Mutex<Sender<String>>>,
) -> Result<GitRequest, GitError> {
    let request = GitRequest::read_git_request(stream);
    match request {
        Ok(request) => {
            let message = format!("{}{:?}", signature, request);
            log_message(&tx, &message);
            Ok(request)
        }
        Err(e) => {
            let message = format!("{}Error al procesar la petición: {}", signature, e);
            log_message(&tx, &message);
            log_client_disconnection_error(&tx, &signature);
            Err(e.into())
        }
    }
}

/// Procesa una solicitud recibida del cliente.
///
/// # Arguments
///
/// * `stream` - Un mutable de referencia a la conexión TCP del cliente.
/// * `tx` - Un Arc de un Mutex que contiene el transmisor para enviar mensajes de registro.
/// * `signature` - Una referencia a la cadena que representa la firma del cliente.
/// * `request` - Una referencia a la solicitud `GitRequest` recibida.
/// * `root_directory` - Una cadena que representa el directorio raíz.
///
/// # Returns
///
/// Retorna un `Result` que contiene `()` en caso de éxito o un `GitError` en caso de fallo.
/// 
fn process_request(
    stream: &mut TcpStream,
    tx: &Arc<Mutex<Sender<String>>>,
    signature: &String,
    request: &GitRequest,
    root_directory: &str,
) -> Result<(), GitError> {
    match request.execute(stream, root_directory) {
        Ok(result) => {
            let message = format!("{}{}", signature, result);
            log_message(tx, &message);
            let message = format!("{}Request exitosa", signature);
            log_message(tx, &message);
            Ok(())
        }
        Err(e) => {
            let message: String = format!("{}Error al procesar la petición: {}", signature, e);
            log_message(tx, &message);
            log_client_disconnection_error(tx, signature);
            Err(e.into())
        }
    }
}

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


/// Acepta conexiones entrantes y maneja cada cliente en un hilo separado.
///
/// # Arguments
///
/// * `listener` - Una referencia al escuchador de TCP.
/// * `shared_tx` - Un `Arc<Mutex<Sender<String>>>` para enviar mensajes de registro.
/// * `src` - Una cadena que representa el directorio fuente.
/// * `handler` - Una función que maneja la conexión del cliente.
///
/// # Returns
///
/// Retorna un `Result` que contiene un vector de `JoinHandle<()>` en caso de éxito o un `GitError` en caso de fallo.
/// 
fn receive_client(
    listener: &TcpListener,
    shared_tx: Arc<Mutex<Sender<String>>>,
    src: &str,
    handler: fn(&mut TcpStream, Arc<Mutex<Sender<String>>>, String) -> Result<(), GitError>,
) -> Result<Vec<JoinHandle<()>>, GitError> {
    // let shared_tx = Arc::new(Mutex::new(tx));
    let mut handles: Vec<JoinHandle<()>> = vec![];
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let tx = Arc::clone(&shared_tx);
                println!("Nueva conexión: {:?}", stream.local_addr());
                let root_directory = src.to_string().clone();
                handles.push(std::thread::spawn(move || {
                    let _ = handler(&mut stream, tx, root_directory);
                }));
            }
            Err(e) => {
                eprintln!("Error al aceptar la conexión: {}", e);
            }
        }
    }
    Ok(handles)
}

fn handle_client_http(
    _stream: &mut TcpStream,
    _tx: Arc<Mutex<Sender<String>>>,
    _root_directory: String,
) -> Result<(), GitError> {
    print!("HTTP");
    Ok(())
}

fn initialize_config() -> Result<Config, GitError> {
    let args: Vec<String> = env::args().collect();
    Config::new(args)
}

fn create_listener(ip: &str, port: &String) -> Result<TcpListener, GitError> {
    let address = format!("{}:{}", ip, port);
    Ok(start_server(&address)?)
}

fn start_logging(path_log: String) -> Result<(Arc<Mutex<Sender<String>>>, JoinHandle<()>), GitError> {
    let (tx, rx) = mpsc::channel();
    let shared_tx = Arc::new(Mutex::new(tx));
    let log_handle = thread::spawn(move || {
        let _ = handle_log_file(&path_log, rx);
    });
    Ok((shared_tx, log_handle))
}

fn start_server_thread(
    listener: TcpListener,
    shared_tx: Arc<Mutex<Sender<String>>>,
    src: String,
    handler: fn(&mut TcpStream, Arc<Mutex<Sender<String>>>, String) -> Result<(), GitError>,
) -> Result<JoinHandle<()>, GitError> {
    let handle = thread::spawn(move || {
        let _ = receive_client(&listener, shared_tx, &src, handler);
    });
    Ok(handle)
}

fn wait_for_threads(log_handle: JoinHandle<()>, daemon_handle: JoinHandle<()>, http_handle: JoinHandle<()>) {
    log_handle.join().expect("No se pudo escribir el archivo de log");
    daemon_handle.join().expect("No hay clientes en git-daemon");
    http_handle.join().expect("No hay clientes en HTTP");
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