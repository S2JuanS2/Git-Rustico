use git::config::Config;
use git::errors::GitError;
use git::git_transport::git_request::GitRequest;
use git::util::connections::start_server;
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

fn main() -> Result<(), GitError> {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(args)?;
    print!("{}", config);

    let address_daemon = format!("{}:{}", config.ip, config.port_daemon);
    let listener_daemon = start_server(&address_daemon)?;

    let address_http = format!("{}:{}", config.ip, config.port_http);
    let listener_http: TcpListener = start_server(&address_http)?;

    let (tx, rx) = mpsc::channel();
    let shared_tx = Arc::new(Mutex::new(tx));

    let log: JoinHandle<()> = thread::spawn(move || {
        let _ = handle_log_file(&config.path_log, rx);
    });

    let src = config.src.clone();
    let tx_daemon = Arc::clone(&shared_tx);
    let clients = thread::spawn(move || {
        let _ = receive_client(&listener_daemon, tx_daemon, &src, handle_client_daemon);
    });

    let src = config.src.clone();
    let tx_http = Arc::clone(&shared_tx);
    let clients_http = thread::spawn(move || {
        let _ = receive_client(&listener_http, tx_http, &src, handle_client_http);
    });

    
    clients_http.join().expect("No hay clientes en HTTP");
    clients.join().expect("No hay clientes en git-daemon");
    log.join().expect("No se pudo escribir el archivo de log");

    Ok(())
}


/// Acepta conexiones entrantes y maneja cada cliente en un hilo separado.
///
/// # Arguments
///
/// * `listener` - Una referencia al escuchador de TCP.
/// * `tx` - El transmisor para enviar mensajes de registro.
/// * `src` - Una cadena que representa el directorio fuente.
///
/// # Returns
///
/// Retorna un `Result` que contiene un vector de `JoinHandle<()>` en caso de éxito o un `GitError` en caso de fallo.
/// 
// fn receive_client_daemon(
//     listener: &TcpListener,
//     tx: Sender<String>,
//     src: &str,
// ) -> Result<Vec<JoinHandle<()>>, GitError> {
//     let shared_tx = Arc::new(Mutex::new(tx));
//     let mut handles: Vec<JoinHandle<()>> = vec![];
//     for stream in listener.incoming() {
//         match stream {
//             Ok(mut stream) => {
//                 let tx = Arc::clone(&shared_tx);
//                 println!("Nueva conexión: {:?}", stream.local_addr());
//                 let root_directory = src.to_string().clone();
//                 handles.push(std::thread::spawn(move || {
//                     let _ = handle_client_daemon(&mut stream, tx, root_directory);
//                 }));
//             }
//             Err(e) => {
//                 eprintln!("Error al aceptar la conexión: {}", e);
//             }
//         }
//     }
//     Ok(handles)
// }

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