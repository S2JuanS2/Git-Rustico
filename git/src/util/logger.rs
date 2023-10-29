use std::net::TcpStream;
use std::sync::mpsc::Receiver;
use std::sync::{Mutex, Arc, mpsc::Sender};
use std::fs::OpenOptions;
use std::io::Write;
use crate::errors::GitError;

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
pub fn log_message(tx: &Arc<Mutex<Sender<String>>>, message: &str) -> Result<(), GitError> {
    match send_message_channel(tx, message, GitError::GenericError)
    {
        Ok(_) => (),
        Err(_) => eprintln!("Fallo al escribir en el logger: {}", message),
    };
    Ok(())
}

pub fn write_log_file(log_path: &str, rx: Receiver<String>) -> Result<(), std::io::Error> {
    // Intentamos abrir el archivo en modo append o crearlo si no existe.
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;

    // Creamos un bucle para recibir datos del canal y escribirlos en el archivo.
    for received_data in rx {
        writeln!(file, "{}", received_data)?;
    }

    // Cerramos el archivo al finalizar.
    file.sync_all()?;
    Ok(())
}


pub fn new_client_logger(
    stream: &TcpStream,
    tx: &Arc<Mutex<Sender<String>>>,
){
    match stream.peer_addr() {
        Ok(addr) => {
            let message = format!("Conexión establecida con {}", addr);
            let _ = log_message(tx, &message);
        }
        Err(_) => {
            let _ = log_message(tx, "Cliente desconocido conectado");
        },
    };
}

pub fn get_client_signature(stream: &TcpStream) -> Result<String, GitError> {
    match stream.peer_addr() {
        Ok(addr) => Ok(format!("Client {} => ", addr)),
        Err(_) => Ok("Cliente desconocido => ".to_string())
    }
}