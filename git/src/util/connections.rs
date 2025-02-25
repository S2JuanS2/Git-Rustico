use crate::consts::FLUSH_PKT;
use crate::consts::PKT_DONE;
use crate::consts::WANT;
use crate::git_server::GitServer;
use crate::git_transport::negotiation::receive_nak;
use crate::git_transport::negotiation::upload_request_type;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;

use super::errors::UtilError;
use super::objects::ObjectEntry;
use super::packfile::read_packfile_data;
use super::packfile::read_packfile_header;

/// Inicia una conexión de cliente con el servidor en la dirección IP proporcionada.
///
/// # Argumentos
/// - `Address`: Una cadena de texto que representa la address del servidor al que se desea conectar.
///
/// # Retorno
/// Un Result que indica si la conexión de cliente se estableció con éxito (Ok) o si se
/// produjo un error (Err) de UtilError, como un error de conexión.
pub fn start_client(address: &str) -> Result<TcpStream, UtilError> {
    match TcpStream::connect(address) {
        Ok(socket) => Ok(socket),
        Err(_) => Err(UtilError::ClientConnection),
    }
}

/// Realiza la negociación del paquete (packfile) enviando una solicitud al servidor con las
/// referencias anunciadas y los datos de capacidad, y luego procesa las respuestas del servidor.
///
/// # Argumentos
/// - `socket`: Un TcpStream que representa la conexión con el servidor.
/// - `advertised`: Un vector de AdvertisedRefLine que contiene las referencias anunciadas por el servidor.
///
/// # Retorno
/// Un Result que indica si la negociación del paquete se realizó con éxito (Ok) o si se
/// produjo un error (Err) de UtilError.
pub fn packfile_negotiation(
    socket: &mut TcpStream,
    git_server: &GitServer,
) -> Result<(), UtilError> {
    let refs = git_server.get_references();
    upload_request_type(socket, refs, WANT)?;
    send_done(socket, UtilError::UploadRequestDone)?;
    receive_nak(socket)?;
    Ok(())
}

pub fn receive_packfile(socket: &mut TcpStream) -> Result<Vec<(ObjectEntry, Vec<u8>)>, UtilError> {
    // read_pack_prueba(socket)?;
    let objects = read_packfile_header(socket)?;
    println!("Objects: {}", objects);
    read_packfile_data(socket, objects as usize)
}

/// Envía un mensaje a través de un socket a un servidor.
///
/// Esta función toma un socket mutable y un mensaje en forma de cadena y lo envía al servidor.
/// Se asegura del envio con un flush
///
/// # Argumentos
///
/// - `socket`: Una mutable referencia a un objeto que implementa el trait `Write` para la salida de datos.
/// - `message`: El mensaje que se va a enviar al servidor en forma de cadena.
/// - `error`: Error que se devolvera si falla el write
///
/// # Retorno
///
/// - `Result<(), UtilError>`: Un resultado que indica si la operación fue exitosa o si se produjo un error.
///   Si la operación se realiza con éxito, se devuelve `Ok(())`. Si se produce un error, se devuelve un error pasado por parametro.
///
pub fn send_message(
    socket: &mut dyn Write,
    message: &str,
    error: UtilError,
) -> Result<(), UtilError> {
    if socket.write(message.as_bytes()).is_err() {
        return Err(error);
    };

    if socket.flush().is_err() {
        return Err(error);
    };
    Ok(())
}

pub fn send_bytes(writer: &mut dyn Write, bytes: &[u8], error: UtilError) -> Result<(), UtilError> {
    if writer.write_all(bytes).is_err() {
        return Err(error);
    };

    if writer.flush().is_err() {
        return Err(error);
    };
    Ok(())
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
/// # Retorno
///
/// - `Result<(), UtilError>`: Un resultado que indica si la operación fue exitosa o si se produjo un error.
///   Si la operación se realiza con éxito, se devuelve `Ok(())`. Si se produce un error, se devuelve un error `UtilError`.
pub fn send_flush(socket: &mut dyn Write, error: UtilError) -> Result<(), UtilError> {
    send_message(socket, FLUSH_PKT, error)
}

/// Envia un mensaje "done" al servidor a través del socket proporcionado.
///
/// # Argumentos
/// - `socket`: Un objeto que implementa el trait `Write`, como un socket o un archivo, que
///   se utiliza para enviar el mensaje "done" al servidor.
/// - `error`: Un valor de tipo `UtilError` que se utilizará en caso de error durante el envío del mensaje.
///
/// # Retorno
/// Un Result que indica si el envío del mensaje "done" se realizó con éxito (Ok) o si se
/// produjo un error (Err) de UtilError.
pub fn send_done(socket: &mut dyn Write, error: UtilError) -> Result<(), UtilError> {
    send_message(socket, PKT_DONE, error)
}

pub fn received_message(
    stream: &mut dyn Read,
    message: &str,
    error: UtilError,
) -> Result<(), UtilError> {
    let mut buffer = vec![0u8; message.len()];
    if stream.read_exact(&mut buffer).is_err() {
        return Err(UtilError::PackfileNegotiationReceiveNAK);
    }
    let response = String::from_utf8_lossy(&buffer);

    if response != message {
        return Err(error);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::{FLUSH_PKT, PKT_NAK};
    use std::io::Cursor;

    #[test]
    fn send_message_sends_data_to_socket() {
        let mut socket = Cursor::new(vec![]);

        let message = "Hello, Git!".to_string();
        let result = send_message(&mut socket, &message, UtilError::GenericError);

        assert!(result.is_ok());

        let written_data = socket.into_inner();
        let received_message = String::from_utf8_lossy(&written_data);
        assert_eq!(received_message, message);
    }

    #[test]
    fn send_flush_sends_flush_pkt() {
        let mut socket = Cursor::new(vec![]);

        let result = send_flush(&mut socket, UtilError::GenericError);

        assert!(result.is_ok());

        let written_data = socket.into_inner();
        let received_flush_pkt = String::from_utf8_lossy(&written_data);
        assert_eq!(received_flush_pkt, FLUSH_PKT);
    }

    #[test]
    fn test_received_message_success() {
        let message = PKT_NAK;
        let mut stream = Cursor::new(message.as_bytes().to_vec());

        let result = received_message(&mut stream, message, UtilError::GenericError);
        assert!(result.is_ok());
    }

    #[test]
    fn test_received_message_fail_response() {
        let message = "Test message";
        let response = "Different message";
        let mut stream = Cursor::new(response.as_bytes().to_vec());

        let result = received_message(&mut stream, message, UtilError::GenericError);
        assert!(result.is_err());
    }

    #[test]
    fn test_received_message_fail_buffer_read() {
        let message = "Test message";
        let mut stream = Cursor::new(vec![]); // Simulate an empty stream

        let result = received_message(&mut stream, message, UtilError::GenericError);
        assert!(result.is_err());
    }
}
