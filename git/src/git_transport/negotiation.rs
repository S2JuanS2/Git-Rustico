
use crate::{consts::PKT_NACK, util::{errors::UtilError, pkt_line, connections::{send_message, send_flush}, validation::is_valid_obj_id}};
use std::{io::Read, net::TcpStream};

use super::advertised::AdvertisedRefs;

/// Realiza una solicitud de carga al servidor Git.
///
/// Esta función toma un socket `TcpStream` y una lista de `AdvertisedRefLine` que
/// han sido anunciadas por el servidor. Examina cada referencia y, en caso de que
/// sea una referencia, crea y envía un mensaje "want" al servidor. Estos mensajes "want"
/// le indican al servidor qué objetos específicos necesita el cliente para completar
/// su trabajo.
///
/// # Argumentos
///
/// - `socket`: Un `TcpStream` abierto para comunicarse con el servidor Git.
/// - `advertised`: Un vector de `AdvertisedRefLine` que contiene las referencias
/// anunciadas por el servidor.
///
/// # Errores
///
/// Esta función devuelve un `Result`. Si ocurre un error al enviar la solicitud
/// de carga, se devuelve un error de tipo `UtilError` específico.
///
/// En este ejemplo, se inicia una conexión al servidor Git y se envía una solicitud de carga para la referencia dada.
///
/// # Retorno
///
/// Esta función no devuelve ningún valor. Si se completa con éxito, indica que las solicitudes "want" se han enviado al servidor correctamente.
pub fn upload_request(
    socket: &mut TcpStream,
    advertised: &AdvertisedRefs,
) -> Result<(), UtilError> {
    for refs in advertised.get_references() {
        let message = format!("want {}\n", refs.get_hash());
        let message = pkt_line::add_length_prefix(&message, message.len());
        send_message(socket, message, UtilError::UploadRequest)?;
    }
    // println!();
    send_flush(socket, UtilError::UploadRequestFlush)?;
    Ok(())
}

pub fn receive_nack(stream: &mut dyn Read) -> Result<(), UtilError> {
    let mut buffer = [0u8; 8]; // Tamaño suficiente para "0008NAK\n"
    if stream.read_exact(&mut buffer).is_err() {
        return Err(UtilError::PackfileNegotiationReceiveNACK);
    }
    let response = String::from_utf8_lossy(&buffer);

    if response != PKT_NACK {
        return Err(UtilError::PackfileNegotiationReceiveNACK);
    }
    Ok(())
}

pub fn receive_request_command(stream: &mut dyn Read) -> Result<Vec<String>, UtilError> {
    let mut want = Vec::new();
    let lines = pkt_line::read(stream)?;

    // Want
    for i in 0..lines.len() {
        let line = String::from_utf8_lossy(&lines[i]);
        if !line.starts_with("want") {
            return Err(UtilError::UnexpectedRequestNotWant);
        }
        let request = line.trim().to_string();
        let mut iter = request.split_ascii_whitespace();
        iter.next();
        let hash = match iter.next()
        {
            Some(hash) => hash,
            None => return Err(UtilError::InvalidRequestFormat(request.to_string())),
        };
        if !is_valid_obj_id(hash)
        {
            return Err(UtilError::InvalidObjectId);
        }
        want.push(hash.to_string());
    }

    // Done?
    let lines = pkt_line::read(stream)?;
    if lines.len() == 1 
    {
        let done = String::from_utf8_lossy(&lines[0]);
        if done != "done" {
            return Err(UtilError::InvalidRequestFormat(done.to_string()));
        }
        return Ok(want);
    }

    // Have
    let lines = pkt_line::read(stream)?;
    for i in 0..lines.len() {
        let line = String::from_utf8_lossy(&lines[i]);
        if !line.starts_with("have") {
            return Err(UtilError::UnexpectedRequestNotWant);
        }
        let request = line.trim().to_string();
        let mut iter = request.split_ascii_whitespace();
        iter.next();
        let hash = match iter.next()
        {
            Some(hash) => hash,
            None => return Err(UtilError::InvalidRequestFormat(request.to_string())),
        };
        if !is_valid_obj_id(hash)
        {
            return Err(UtilError::InvalidObjectId);
        }
        want.push(hash.to_string());
    }
    
    // Done
    // let lines = pkt_line::read(stream)?;
    // if lines.len() != 1 
    // {
    //     return Err(UtilError::InvalidRequestFormat(done.to_string()));
    // }
    // let done = String::from_utf8_lossy(&lines[0]);
    // if done != "done" {
    //     return Err(UtilError::InvalidRequestFormat(done.to_string()));
    // }
    Ok(want)
}
