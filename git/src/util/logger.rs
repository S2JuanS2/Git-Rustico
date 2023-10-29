use std::net::TcpStream;
use std::sync::mpsc::Receiver;
use std::sync::{Mutex, Arc, mpsc::Sender};
use std::fs::{OpenOptions, File};
use std::io::Write;
use crate::errors::GitError;

use super::log_file::LogFile;


/// Envía un mensaje a través del canal con un transmisor protegido por Mutex.
///
/// # Argumentos
///
/// * `tx`: Un puntero a un canal (`Sender`) envuelto en un Mutex y Arc para enviar mensajes.
/// * `message`: Un string que contiene el mensaje a enviar a través del canal.
/// * `error`: El tipo de error (`GitError`) a devolver si hay un fallo al enviar el mensaje.
///
/// # Retorno
///
/// Retorna un `Result` indicando si se envió el mensaje correctamente o si ocurrió un error.
///
/// Si el mensaje se envía con éxito, se devuelve `Ok(())`.
/// Si hay un error al enviar el mensaje, se devuelve un `Err` con el tipo de error (`GitError`).
/// 
fn send_message_channel(
    tx: &Arc<Mutex<Sender<String>>>,
    message: &str,
    error: GitError,
) -> Result<(), GitError> {
    match tx.lock().unwrap().send(message.to_string()) {
        Ok(_) => Ok(()),
        Err(_) => Err(error),
    }
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
    match send_message_channel(tx, message, GitError::GenericError)
    {
        Ok(_) => (),
        Err(_) => eprintln!("Fallo al escribir en el logger: {}", message),
    };
}


pub fn handle_log_file(log_path: &str, rx: Receiver<String>) -> Result<(), GitError> {
    let mut file = LogFile::new(log_path)?;

    // Creamos un bucle para recibir datos del canal y escribirlos en el archivo.
    for received_data in rx {
        if writeln!(file, "{}", received_data).is_err() {
            println!("Error al escribir en el logger: {}", received_data);
        }
        if file.flush().is_err()
        {
            eprintln!("Error al sincronizar el archivo de log con el siguiente mensaje: {}", received_data);
        }
    }

    // Cerramos el archivo al finalizar.
    file.sync_all()
}


pub fn log_client_connect(
    stream: &TcpStream,
    tx: &Arc<Mutex<Sender<String>>>,
){
    match stream.peer_addr() {
        Ok(addr) => {
            let message = format!("Conexión establecida con {}", addr);
            log_message(tx, &message);
        }
        Err(_) => {
            log_message(tx, "Cliente desconocido conectado");
        },
    };
}

pub fn get_client_signature(stream: &TcpStream) -> Result<String, GitError> {
    match stream.peer_addr() {
        Ok(addr) => Ok(format!("Client {} => ", addr)),
        Err(_) => Ok("Cliente desconocido => ".to_string())
    }
}

pub fn log_client_disconnection(tx: &Arc<Mutex<Sender<String>>>, signature: &str) {
    let message = format!("{}Conexión terminada", signature);
    log_message(&tx, &message)
}