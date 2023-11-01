use super::{
    advertised::AdvertisedRefs,
    connections::{send_flush, send_message},
    pkt_line, errors::UtilError,
};
use crate::{consts::NACK, errors::GitError};
use std::{io::Read, net::TcpStream};

/// Realiza una solicitud de carga al servidor Git.
///
/// Esta función toma un socket `TcpStream` y una lista de `AdvertisedRefs` que
/// han sido anunciadas por el servidor. Examina cada referencia y, en caso de que
/// sea una referencia, crea y envía un mensaje "want" al servidor. Estos mensajes "want"
/// le indican al servidor qué objetos específicos necesita el cliente para completar
/// su trabajo.
///
/// # Argumentos
///
/// - `socket`: Un `TcpStream` abierto para comunicarse con el servidor Git.
/// - `advertised`: Un vector de `AdvertisedRefs` que contiene las referencias
/// anunciadas por el servidor.
///
/// # Errores
///
/// Esta función devuelve un `Result`. Si ocurre un error al enviar la solicitud
/// de carga, se devuelve un error de tipo `GitError` específico.
///
/// En este ejemplo, se inicia una conexión al servidor Git y se envía una solicitud de carga para la referencia dada.
///
/// # Retorno
///
/// Esta función no devuelve ningún valor. Si se completa con éxito, indica que las solicitudes "want" se han enviado al servidor correctamente.
pub fn upload_request(
    socket: &mut TcpStream,
    advertised: Vec<AdvertisedRefs>,
) -> Result<(), UtilError> {
    for a in advertised {
        let message = match a {
            AdvertisedRefs::Ref {
                obj_id,
                ref_name: _,
            } => format!("want {}\n", obj_id),
            _ => continue,
        };
        let message = pkt_line::add_length_prefix(&message, message.len());
        send_message(socket, message, UtilError::UploadRequest)?;
    }
    // println!();
    send_flush(socket, UtilError::UploadRequestFlush)?;
    Ok(())
}

pub fn receive_nack(stream: &mut dyn Read) -> Result<(), GitError> {
    let mut buffer = [0u8; 8]; // Tamaño suficiente para "0008NAK\n"
    if stream.read_exact(&mut buffer).is_err() {
        return Err(GitError::PackfileNegotiationNACK);
    }
    let response = String::from_utf8_lossy(&buffer);

    if response != NACK {
        return Err(GitError::PackfileNegotiationNACK);
    }
    Ok(())
}
