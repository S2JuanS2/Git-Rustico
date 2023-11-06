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
    let tx = match tx.lock()
    {
        Ok(tx) => tx,
        Err(_) => return Err(error),
    };
    match tx.send(message.to_string()) {
        Ok(_) => Ok(()),
        Err(_) => Err(error),
    }
}

pub fn write_client_log(directory: &str, content: String, path_log: &str) -> Result<(), GitError> {
    let dir_path = format!("{}/{}", directory, GIT_DIR);
    create_directory(Path::new(&dir_path))?;
    let log_path = format!("{}/{}/{}", directory, GIT_DIR, path_log);

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(log_path);

    let mut file = match file {
        Ok(file) => file,
        Err(_) => return Err(GitError::OpenFileError),
    };

    if writeln!(file, "Client => {}", content).is_err() {
        return Err(GitError::WriteFileError);
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
///
pub fn log_client_connect(stream: &TcpStream, tx: &Arc<Mutex<Sender<String>>>) {
    match stream.peer_addr() {
        Ok(addr) => {
            let message = format!("Conexión establecida con {}", addr);
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
/// # Errores
///
/// Devuelve un error si no se puede obtener la dirección del cliente o si ocurre algún problema.
///
pub fn get_client_signature(stream: &TcpStream) -> Result<String, UtilError> {
    match stream.peer_addr() {
        Ok(addr) => Ok(format!("Client {} => ", addr)),
        Err(_) => Ok("Cliente desconocido => ".to_string()),
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

pub fn log_client_disconnection_error(tx: &Arc<Mutex<Sender<String>>>, signature: &str) {
    let message = format!("{}Conexión terminada por error", signature);
    log_message(tx, &message)
}

pub fn log_client_disconnection_success(tx: &Arc<Mutex<Sender<String>>>, signature: &str) {
    let message = format!("{}Conexión terminada", signature);
    log_message(tx, &message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn test_log_client_disconnection() {
        let (tx, rx) = mpsc::channel();
        let arc = Arc::new(Mutex::new(tx));

        log_client_disconnection(&arc, "Test: ");

        let received_message = rx.recv().unwrap();
        assert_eq!(received_message, "Test: Conexión terminada");
    }
}
