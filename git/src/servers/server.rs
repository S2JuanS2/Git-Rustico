use std::net::{TcpListener, TcpStream};

use crate::config::Config;
use crate::errors::GitError;
use crate::git_transport::git_request::GitRequest;
use crate::util::logger::{handle_log_file, log_client_disconnection_error, log_message};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Sender};
use std::thread::JoinHandle;
use std::{env, thread};

use super::errors::ServerError;

/// Inicia un servidor en la dirección IP y puerto proporcionados.
///
/// # Argumentos
/// - `ip`: Una cadena de texto que representa la dirección IP y puerto en los que se
///   debe iniciar el servidor en el formato "ip:puerto".
///
/// # Retorno
/// Un Result que indica si el servidor se inició con éxito (Ok) y devuelve un TcpListener para
/// aceptar conexiones entrantes, o si se produjo un error (Err) de UtilError, como un error de conexión.
pub fn start_server(ip: &str) -> Result<TcpListener, ServerError> {
    match TcpListener::bind(ip) {
        Ok(listener) => Ok(listener),
        Err(_) => Err(ServerError::ServerConnection),
    }
}

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
pub fn receive_request(
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
pub fn process_request(
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
pub fn receive_client(
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
                let message = format!("Error al aceptar la conexión: {}", e);
                log_message(&shared_tx, &message);
                eprintln!("Error al aceptar la conexión: {}", e);
            }
        }
    }
    Ok(handles)
}


/// Inicializa la configuración del programa a partir de los argumentos de línea de comandos.
///
/// # Returns
///
/// Retorna un `Result` que contiene la configuración inicializada si es exitosa, o un 
/// `GitError` si falla.
///
/// # Errors
///
/// Puede fallar si hay errores al procesar los argumentos del archivo config.
///
pub fn initialize_config() -> Result<Config, GitError> {
    let args: Vec<String> = env::args().collect();
    Config::new(args)
}

/// Crea y devuelve un `TcpListener` para un servidor escuchando en la dirección especificada por `ip` y `port`.
///
/// # Arguments
///
/// * `ip` - La dirección IP donde se iniciará el servidor.
/// * `port` - El puerto en el que escuchará el servidor, representado como una cadena.
///
/// # Returns
///
/// Retorna un `Result` que contiene un `TcpListener` si la creación es exitosa, o un `GitError` si falla.
///
/// # Errors
///
/// Puede fallar si hay errores al intentar iniciar el servidor en la dirección y puerto especificados.
///
pub fn create_listener(ip: &str, port: &String) -> Result<TcpListener, GitError> {
    let address = format!("{}:{}", ip, port);
    Ok(start_server(&address)?)
}

/// Inicia el logging en un hilo separado, escribiendo en el archivo de log especificado por `path_log`.
///
/// # Arguments
///
/// * `path_log` - La ruta al archivo de log donde se escribirán los registros.
///
/// # Returns
///
/// Retorna un `Result` que contiene una tupla con un `Arc<Mutex<Sender<String>>>` para transmitir mensajes de log
/// y un `JoinHandle<()>` que representa el handle del hilo de logging.
///
/// # Errors
///
/// Puede fallar si hay errores al intentar iniciar el hilo de logging o al abrir el archivo de log.
///
pub fn start_logging(path_log: String) -> Result<(Arc<Mutex<Sender<String>>>, JoinHandle<()>), GitError> {
    let (tx, rx) = mpsc::channel();
    let shared_tx = Arc::new(Mutex::new(tx));
    let log_handle = thread::spawn(move || {
        let _ = handle_log_file(&path_log, rx);
    });
    Ok((shared_tx, log_handle))
}

/// Inicia un hilo para manejar conexiones entrantes en un servidor TCP, utilizando un 
/// `TcpListener` dado.
///
/// # Arguments
///
/// * `listener` - El `TcpListener` que acepta conexiones entrantes.
/// * `shared_tx` - Un `Arc<Mutex<Sender<String>>>` para transmitir mensajes de log.
/// * `src` - La ruta del directorio raíz para operaciones de servidor.
/// * `handler` - Una función que maneja cada conexión entrante.
///
/// # Returns
///
/// Retorna un `Result` que contiene un `JoinHandle<()>` que representa el handle del hilo de 
/// servidor,
/// o un `GitError` si falla.
///
/// # Errors
///
/// Puede fallar si hay errores al manejar conexiones entrantes o al ejecutar el handler de 
/// conexión.
///
pub fn start_server_thread(
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

/// Espera a que finalicen los hilos de logging, servidor de daemon y servidor HTTP.
///
/// # Arguments
///
/// * `log_handle` - El handle del hilo de logging.
/// * `daemon_handle` - El handle del hilo del servidor de daemon.
/// * `http_handle` - El handle del hilo del servidor HTTP.
///
/// # Panics
///
/// Puede generar un pánico si alguno de los hilos no finaliza correctamente.
///
pub fn wait_for_threads(log_handle: JoinHandle<()>, daemon_handle: JoinHandle<()>, http_handle: JoinHandle<()>) {
    log_handle.join().expect("No se pudo escribir el archivo de log");
    daemon_handle.join().expect("No hay clientes en git-daemon");
    http_handle.join().expect("No hay clientes en HTTP");
}