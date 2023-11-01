use crate::consts::DONE;
use crate::consts::FLUSH_PKT;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;

use super::advertised::AdvertisedRefs;
use super::errors::UtilError;
use super::negotiation::receive_nack;
use super::negotiation::upload_request;
use super::objects::ObjectEntry;
use super::packfile::read_packfile_data;
use super::packfile::read_packfile_header;
use super::pkt_line;

/// Inicia un servidor en la dirección IP y puerto proporcionados.
///
/// # Argumentos
/// - `ip`: Una cadena de texto que representa la dirección IP y puerto en los que se
///   debe iniciar el servidor en el formato "ip:puerto".
///
/// # Retorno
/// Un Result que indica si el servidor se inició con éxito (Ok) y devuelve un TcpListener para
/// aceptar conexiones entrantes, o si se produjo un error (Err) de UtilError, como un error de conexión.
pub fn start_server(ip: &str) -> Result<TcpListener, UtilError> {
    match TcpListener::bind(ip) {
        Ok(listener) => Ok(listener),
        Err(_) => Err(UtilError::ServerConnection),
    }
}

/// Inicia una conexión de cliente con el servidor en la dirección IP proporcionada.
///
/// # Argumentos
/// - `ip`: Una cadena de texto que representa la dirección IP del servidor al que se desea conectar.
///
/// # Retorno
/// Un Result que indica si la conexión de cliente se estableció con éxito (Ok) o si se
/// produjo un error (Err) de UtilError, como un error de conexión.
pub fn start_client(ip: &str) -> Result<TcpStream, UtilError> {
    match TcpStream::connect(ip) {
        Ok(socket) => Ok(socket),
        Err(_) => Err(UtilError::ClientConnection),
    }
}

/// Realiza un proceso de descubrimiento de referencias (refs) enviando un mensaje al servidor
/// a través del socket proporcionado, y luego procesa las líneas recibidas para clasificarlas
/// en una lista de AdvertisedRefs.
///
/// # Argumentos
/// - `socket`: Un TcpStream que representa la conexión con el servidor.
/// - `message`: Un mensaje que se enviará al servidor.
///
/// # Retorno
/// Un Result que contiene un vector de AdvertisedRefs si la operación fue exitosa,
/// o un error de UtilError en caso contrario.
pub fn reference_discovery(
    socket: &mut TcpStream,
    message: String,
) -> Result<Vec<AdvertisedRefs>, UtilError> {
    send_message(socket, message, UtilError::ReferenceDiscovey)?;
    let lines = pkt_line::read(socket)?;
    AdvertisedRefs::classify_vec(&lines)
}

/// Realiza la negociación del paquete (packfile) enviando una solicitud al servidor con las
/// referencias anunciadas y los datos de capacidad, y luego procesa las respuestas del servidor.
///
/// # Argumentos
/// - `socket`: Un TcpStream que representa la conexión con el servidor.
/// - `advertised`: Un vector de AdvertisedRefs que contiene las referencias anunciadas por el servidor.
///
/// # Retorno
/// Un Result que indica si la negociación del paquete se realizó con éxito (Ok) o si se
/// produjo un error (Err) de UtilError.
pub fn packfile_negotiation(
    socket: &mut TcpStream,
    advertised: Vec<AdvertisedRefs>,
) -> Result<(), UtilError> {
    upload_request(socket, advertised)?;
    send_done(socket, UtilError::UploadRequestDone)?;
    receive_nack(socket)?;
    Ok(())
}

pub fn receive_packfile(socket: &mut TcpStream) -> Result<Vec<(ObjectEntry, Vec<u8>)>, UtilError> {
    // read_pack_prueba(socket)?;
    let objects = read_packfile_header(socket)?;
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
    message: String,
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
    send_message(socket, FLUSH_PKT.to_string(), error)
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
    send_message(socket, DONE.to_string(), error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::FLUSH_PKT;
    use std::io::Cursor;

    #[test]
    fn send_message_sends_data_to_socket() {
        let mut socket = Cursor::new(vec![]);

        let message = "Hello, Git!".to_string();
        let result = send_message(&mut socket, message.clone(), UtilError::GenericError);

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
}
