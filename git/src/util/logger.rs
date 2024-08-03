use crate::consts::*;
use crate::errors::GitError;
use std::fs::OpenOptions;
use std::io::Write;
use std::net::TcpStream;
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::sync::{mpsc::Sender, Arc, Mutex};

use super::errors::UtilError;
use super::files::create_directory;
use super::log_output::LogOutput;

/// Envía un mensaje a través del canal con un transmisor protegido por Mutex.
///
/// # Argumentos
///
/// * `tx`: Un puntero a un canal (`Sender`) envuelto en un Mutex y Arc para enviar mensajes.
/// * `message`: Un string que contiene el mensaje a enviar a través del canal.
/// * `error`: El tipo de error (`UtilError`) a devolver si hay un fallo al enviar el mensaje.
///
/// # Retorno
///
/// Retorna un `Result` indicando si se envió el mensaje correctamente o si ocurrió un error.
///
/// Si el mensaje se envía con éxito, se devuelve `Ok(())`.
/// Si hay un error al enviar el mensaje, se devuelve un `Err` con el tipo de error (`UtilError`).
///
fn send_message_channel(
    tx: &Arc<Mutex<Sender<String>>>,
    message: &str,
    error: UtilError,
) -> Result<(), UtilError> {
    let tx = match tx.lock() {
        Ok(tx) => tx,
        Err(_) => return Err(error),
    };
    match tx.send(message.to_string()) {
        Ok(_) => Ok(()),
        Err(_) => Err(error),
    }
}

pub fn write_client_log(directory: &str, content: String, path_log: &str) -> Result<(), GitError> {
    
    let git_dir = format!("{}/{}", directory, GIT_DIR);
    let dir_path = Path::new(&git_dir);
    if dir_path.exists() {
        create_directory(dir_path)?;
        let log_path = format!("{}/{}/{}", directory, GIT_DIR, path_log);

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path);

        let mut file = match file {
            Ok(file) => file,
            Err(_) => return Err(GitError::OpenFileError),
        };

        if writeln!(file, "{} Client => {}", chrono::Local::now(), content).is_err() {
            return Err(GitError::WriteFileError);
        }
    }
    Ok(())
}

/// Registra un mensaje a través del canal de un transmisor protegido por Mutex.
///
/// # Argumentos
///
/// * `tx`: Un puntero a un canal (`Sender`) envuelto en un Mutex y Arc para enviar mensajes.
/// * `message`: Un string que contiene el mensaje a registrar a través del canal.
///
/// La función envía el mensaje especificado al logger a través del canal proporcionado.
/// Si hay un error al enviar el mensaje, se registra un mensaje de error en la consola.
///
pub fn log_message(tx: &Arc<Mutex<Sender<String>>>, message: &str) {
    match send_message_channel(tx, message, UtilError::LogMessageSend) {
        Ok(_) => (),
        Err(_) => eprintln!("Fallo al escribir en el logger: {}", message),
    };
}

/// Maneja el archivo de registro, escribiendo los datos recibidos del canal en el archivo de registro indicado.
///
/// # Argumentos
///
/// * `log_path` - La ruta del archivo de registro.
/// * `rx` - El receptor que recibe los datos que se escribirán en el archivo.
///
/// # Errores
///
/// Devuelve un error si ocurre algún problema al operar con el archivo de registro.
///
pub fn handle_log_file(log_path: &str, rx: Receiver<String>) -> Result<(), UtilError> {
    let mut file = LogOutput::new(log_path)?;

    // Creamos un bucle para recibir datos del canal y escribirlos en el archivo.
    for received_data in rx {
        if writeln!(file, "{}", received_data).is_err() {
            println!("Error al escribir en el logger: {}", received_data);
        }
        if file.flush().is_err() {
            eprintln!(
                "Error al sincronizar el archivo de log con el siguiente mensaje: {}",
                received_data
            );
        }
    }

    // Cerramos el archivo al finalizar.
    file.sync_all()
}

/// Registra el evento de conexión de un cliente.
/// Si la obtención de la dirección del cliente tiene éxito, se registra un mensaje con el formato "Conexión establecida con [dirección]". Si hay un error al obtener la dirección del cliente, se registra un mensaje indicando "Cliente desconocido conectado".
///
/// # Argumentos
///
/// * `stream` - El stream del cliente del que se obtendrá la dirección.
/// * `tx` - Arc Mutex del transmisor del canal para escribir mensajes de registro.
/// * `name_server` - El nombre del servidor al que se conecta el cliente.
///
pub fn log_client_connect(stream: &TcpStream, tx: &Arc<Mutex<Sender<String>>>, name_server: &String) {
    match stream.peer_addr() {
        Ok(addr) => {
            let message = format!("{} Conexión establecida con {}", name_server, addr);
            log_message(tx, &message);
        }
        Err(_) => {
            log_message(tx, "Cliente desconocido conectado");
        }
    };
}

/// Obtiene la firma del cliente conectado, incluyendo su dirección si está disponible.
///
/// Si la obtención de la dirección del cliente tiene éxito, devuelve un `Result` con un `String` que contiene la firma del cliente en el formato "Client [dirección] => ". Si hay un error al obtener la dirección del cliente, se devuelve una firma "Cliente desconocido => ".
///
/// # Argumentos
///
/// * `stream` - El stream del cliente del que se obtendrá la dirección.
///
pub fn get_client_signature(stream: &TcpStream, name_server: &String) -> String {
    match stream.peer_addr() {
        Ok(addr) => format!("{} Client {} => ", name_server, addr),
        Err(_) => "Cliente desconocido => ".to_string(),
    }
}

/// Registra la desconexión del cliente y envía un mensaje al logger con la firma del cliente y el evento de desconexión.
///
/// # Argumentos
///
/// * `tx` - El transmisor para enviar mensajes al logger.
/// * `signature` - La firma del cliente conectado. Se espera que contenga la identificación del cliente y esté formateada como "Client [dirección] => ".
///
pub fn log_client_disconnection(tx: &Arc<Mutex<Sender<String>>>, signature: &str) {
    let message = format!("{}Conexión terminada", signature);
    log_message(tx, &message)
}

/// Registra un mensaje de error de desconexión del cliente.
///
/// Esta función formatea y envía un mensaje de registro indicando que la conexión 
/// con el cliente se terminó debido a un error.
///
/// # Parámetros
///
/// - `tx`: Un `Arc<Mutex<Sender<String>>>` para enviar mensajes de registro.
/// - `signature`: Una cadena que contiene la firma del mensaje.
/// 
pub fn log_client_disconnection_error(tx: &Arc<Mutex<Sender<String>>>, signature: &str) {
    let message = format!("{}Conexión terminada por error", signature);
    log_message(tx, &message)
}

/// Registra un mensaje de desconexión exitosa del cliente.
///
/// Esta función formatea y envía un mensaje de registro indicando que la conexión 
/// con el cliente se terminó exitosamente.
///
/// # Parámetros
///
/// - `tx`: Un `Arc<Mutex<Sender<String>>>` para enviar mensajes de registro.
/// - `signature`: Una cadena que contiene la firma del mensaje.
/// 
pub fn log_client_disconnection_success(tx: &Arc<Mutex<Sender<String>>>, signature: &str) {
    let message = format!("{}Conexión terminada", signature);
    log_message(tx, &message)
}

/// Registra un mensaje de error en una firma especifica
///
/// Esta función formatea y envía un mensaje de registro indicando que hubo un error 
/// en la solicitud de la firma, seguido del mensaje de error específico.
///
/// # Parámetros
///
/// - `error`: Una cadena que contiene el mensaje de error.
/// - `signature`: Una cadena que contiene la firma del mensaje.
/// - `tx`: Un `Arc<Mutex<Sender<String>>>` para enviar mensajes de registro.
/// 
pub fn log_request_error(error: &String, signature: &str,tx: &Arc<Mutex<Sender<String>>>) {
    let message = format!("{}Error en la solicitud.", signature);
    log_message(tx, &message);
    let message = format!("{}Error: {}", signature, error);
    log_message(tx, &message);
}

/// Registra un mensaje con una firma especificada.
///
/// Esta función formatea y envía un mensaje de registro precedido por una firma 
/// específica.
///
/// # Parámetros
///
/// - `tx`: Un `Arc<Mutex<Sender<String>>>` para enviar mensajes de registro.
/// - `signature`: Una cadena que contiene la firma del mensaje.
/// - `message`: Una cadena que contiene el mensaje a registrar.
/// 
pub fn log_message_with_signature(tx: &Arc<Mutex<Sender<String>>>, signature: &str, message: &str) {
    let message = format!("{} {}", signature, message);
    log_message(tx, &message);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    fn setup() -> (Arc<Mutex<mpsc::Sender<String>>>, mpsc::Receiver<String>) {
        let (tx, rx) = mpsc::channel();
        (Arc::new(Mutex::new(tx)), rx)
    }

    #[test]
    fn test_log_client_disconnection() {
        let (tx, rx) = setup();
        log_client_disconnection(&tx, "Test: ");

        let received_message = rx.recv().unwrap();
        assert_eq!(received_message, "Test: Conexión terminada");
    }

    #[test]
    fn test_log_client_disconnection_error() {
        let (tx, rx) = setup();
        let signature = "Client [127.0.0.1:8080] => ";
        log_client_disconnection_error(&tx, signature);
        assert_eq!(rx.recv().unwrap(), "Client [127.0.0.1:8080] => Conexión terminada por error");
    }

    #[test]
    fn test_log_client_disconnection_success() {
        let (tx, rx) = setup();
        let signature = "Client [127.0.0.1:8080] => ";
        log_client_disconnection_success(&tx, signature);
        assert_eq!(rx.recv().unwrap(), "Client [127.0.0.1:8080] => Conexión terminada");
    }

    #[test]
    fn test_log_request_error() {
        let (tx, rx) = setup();
        let signature = "Client [127.0.0.1:8080] => ";
        let error = "404 Not Found".to_string();
        log_request_error(&error, signature, &tx);
        assert_eq!(rx.recv().unwrap(), "Client [127.0.0.1:8080] => Error en la solicitud.");
        assert_eq!(rx.recv().unwrap(), "Client [127.0.0.1:8080] => Error: 404 Not Found");
    }

    #[test]
    fn test_log_message_with_signature() {
        let (tx, rx) = setup();
        let signature = "Client [127.0.0.1:8080] =>";
        let message = "Solicitud recibida";
        log_message_with_signature(&tx, signature, message);
        assert_eq!(rx.recv().unwrap(), "Client [127.0.0.1:8080] => Solicitud recibida");
    }
}
