use super::{advertised::AdvertisedRefs, connections::send_message, pkt_line};
use crate::{consts::NACK, errors::GitError};
use std::{io::Write, net::TcpStream};


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
) -> Result<(), GitError> {
    for a in advertised {
        let message = match a {
            AdvertisedRefs::Ref {
                obj_id,
                ref_name: _,
            } => format!("want {}\n", obj_id),
            _ => continue,
        };
        let message = pkt_line::add_length_prefix(&message, message.len());
        send_message(socket, message, GitError::UploadRequest)?;
    }
    Ok(())
}


pub fn receive_nack(socket: &mut dyn Write) -> Result<(), GitError> {
    send_message(socket, NACK.to_string(), GitError::PackfileNegotiationNACK)
}
