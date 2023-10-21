use crate::consts::FLUSH_PKT;
use crate::errors::GitError;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;

use super::advertised::AdvertisedRefs;
use super::pkt_line;

pub fn start_server(ip: &str) -> Result<TcpListener, GitError> {
    match TcpListener::bind(ip) {
        Ok(listener) => Ok(listener),
        Err(_) => Err(GitError::ServerConnectionError),
    }
}

pub fn start_client(ip: &str) -> Result<TcpStream, GitError> {
    match TcpStream::connect(ip) {
        Ok(socket) => Ok(socket),
        Err(_) => Err(GitError::ClientConnectionError),
    }
}

pub fn reference_discovery(socket: &mut TcpStream, message: String) -> Result<Vec<AdvertisedRefs>, GitError>
{
    send_message(socket, message)?;
    let lines = pkt_line::read(socket)?;
    AdvertisedRefs::classify_vec(&lines)
}


/// Envía un mensaje a través de un socket a un servidor.
///
/// Esta función toma un socket mutable y un mensaje en forma de cadena y lo envía al servidor.
///
/// # Argumentos
///
/// - `socket`: Una mutable referencia a un objeto que implementa el trait `Write` para la salida de datos.
/// - `message`: El mensaje que se va a enviar al servidor en forma de cadena.
///
/// # Retorno
///
/// - `Result<(), GitError>`: Un resultado que indica si la operación fue exitosa o si se produjo un error.
///   Si la operación se realiza con éxito, se devuelve `Ok(())`. Si se produce un error, se devuelve un error `GitError`.
///
pub fn send_message(socket: &mut dyn Write, message: String) -> Result<(), GitError> {
    match socket.write(message.as_bytes()) {
        Ok(_) => Ok(()),
        Err(_) => Err(GitError::GenericError),
    }
}

/// Finaliza la conexión enviando un paquete de finalización al servidor.
///
/// Después de que se hayan descubierto las referencias y las capacidades,
/// el cliente puede decidir terminar la conexión enviando un paquete de finalización,
/// indicando al servidor que puede finalizar de manera segura y desconectarse cuando no necesite
/// más datos de paquetes. Esto puede ocurrir con el comando `ls-remote` y también puede ocurrir
/// cuando el cliente ya está actualizado.
///
/// # Argumentos
///
/// - `socket`: Una mutable referencia a un objeto que implementa el trait `Write` para la salida de datos.
///
/// # Retornoes
///
/// - `Result<(), GitError>`: Un resultado que indica si la operación fue exitosa o si se produjo un error.
///   Si la operación se realiza con éxito, se devuelve `Ok(())`. Si se produce un error, se devuelve un error `GitError`.
pub fn end_connection(socket: &mut dyn Write) -> Result<(), GitError> {
    match socket.write(FLUSH_PKT.to_string().as_bytes()) {
        Ok(_) => Ok(()),
        Err(_) => Err(GitError::GenericError),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use crate::consts::FLUSH_PKT;

    #[test]
    fn send_message_sends_data_to_socket() {
        let mut socket = Cursor::new(vec![]);

        let message = "Hello, Git!".to_string();
        let result = send_message(&mut socket, message.clone());

        assert!(result.is_ok());

        let written_data = socket.into_inner();
        let received_message = String::from_utf8_lossy(&written_data);
        assert_eq!(received_message, message);
    }

    #[test]
    fn end_connection_sends_flush_pkt() {
        let mut socket = Cursor::new(vec![]);

        let result = end_connection(&mut socket);

        assert!(result.is_ok());

        let written_data = socket.into_inner();
        let received_flush_pkt = String::from_utf8_lossy(&written_data);
        assert_eq!(received_flush_pkt, FLUSH_PKT);
    }
}
