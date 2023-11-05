
use crate::{consts::{PKT_NACK, PKT_DONE}, util::{errors::UtilError, pkt_line, connections::{send_message, send_flush, received_message}, validation::is_valid_obj_id}};
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

pub fn receive_request(stream: &mut dyn Read) -> Result<(Vec<String>, Vec<String>), UtilError> {
    let lines = pkt_line::read(stream)?;

    let (capacilities, request) = process_received_requests(lines)?;

    // Done
    received_message(stream, PKT_DONE, UtilError::NegociacionExpectedDone)?;
    Ok((capacilities, request))
}

fn process_received_requests(lines: Vec<Vec<u8>>) -> Result<(Vec<String>, Vec<String>), UtilError>
{
    let mut request = Vec::new();

    // Want and capabilities
    let (hash, capacilities) = extraction_capabilities(&lines[0])?;
    request.push(hash);

    // Want
    let want = receive_request_type(lines[1..].to_vec(), "want")?;
    request.extend(want);

    Ok((capacilities, request))
}

fn extraction_capabilities(line: &Vec<u8>) -> Result<(String, Vec<String>), UtilError> {
    let line_str = String::from_utf8_lossy(&line);
    let mut line_split = line_str.split_ascii_whitespace();
    let type_request = line_split.next().ok_or_else(|| UtilError::InvalidRequestFormat(line_str.to_string()))?;
    if type_request != "want" {
        return Err(UtilError::UnexpectedRequestNotWant);
    }
    let hash = line_split.next().ok_or_else(|| UtilError::InvalidRequestFormat(line_str.to_string()))?;
    if !is_valid_obj_id(hash)
    {
        return Err(UtilError::InvalidObjectId);
    }
    let capacilities = line_split.collect::<Vec<&str>>().iter().map(|s| s.to_string()).collect::<Vec<String>>();
    Ok((hash.to_string(), capacilities))
}


fn receive_request_type(lines: Vec<Vec<u8>>, type_req: &str) -> Result<Vec<String>, UtilError>
{
    lines.iter().try_fold(Vec::new(), |mut acc, line| {
        let line_str = String::from_utf8_lossy(line);
    
        if !line_str.starts_with(type_req) {
            return Err(UtilError::UnexpectedRequestNotWant);
        }
    
        let request = line_str.trim().to_string();
        let hash = request.split_ascii_whitespace().nth(1).ok_or_else(|| UtilError::InvalidRequestFormat(request.to_string()))?;
    
        if !is_valid_obj_id(hash) {
            return Err(UtilError::InvalidObjectId);
        }
    
        acc.push(hash.to_string());
        Ok(acc)
    })
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_receive_request_type_valid_want() {
        let mut lines = Vec::new();
        lines.push(b"want 74730d410fcb6603ace96f1dc55ea6196122532d".to_vec());
        lines.push(b"want 7d1665144a3a975c05f1f43902ddaf084e784dbe".to_vec());
        lines.push(b"want 5a3f6be755bbb7deae50065988cbfa1ffa9ab68a".to_vec());
        let result = receive_request_type( lines, "want");
        assert!(result.is_ok());
        let wanted_hashes = result.unwrap();
        assert_eq!(wanted_hashes, vec!["74730d410fcb6603ace96f1dc55ea6196122532d", "7d1665144a3a975c05f1f43902ddaf084e784dbe", "5a3f6be755bbb7deae50065988cbfa1ffa9ab68a"]);
    }

    #[test]
    fn test_receive_request_type_valid_have() {
        let mut lines = Vec::new();
        lines.push(b"have 7e47fe2bd8d01d481f44d7af0531bd93d3b21c01".to_vec());
        lines.push(b"have 74730d410fcb6603ace96f1dc55ea6196122532d".to_vec());
        let result = receive_request_type(lines, "have");
        assert!(result.is_ok());
        let have_hashes = result.unwrap();
        assert_eq!(have_hashes, vec!["7e47fe2bd8d01d481f44d7af0531bd93d3b21c01", "74730d410fcb6603ace96f1dc55ea6196122532d"]);
    }

    #[test]
    fn test_receive_request_type_invalid() {
        let mut lines = Vec::new();
        lines.push(b"have 74730d410fcb6603ace96f1dc55ea6196122532d".to_vec());
        lines.push(b"want 7d1665144a3a975c05f1f43902ddaf084e784dbe".to_vec());
        lines.push(b"have 5a3f6be755bbb7deae50065988cbfa1ffa9ab68a".to_vec());
        let result = receive_request_type(lines, "want");
        assert!(result.is_err());
    }

    #[test]
    fn test_receive_request_empty() {
        let  lines = Vec::new();
        let result = receive_request_type(lines, "want");
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_extraction_capabilities_valid() {
        let line = b"want 74730d410fcb6603ace96f1dc55ea6196122532d capability1 capability2 capability3\n".to_vec();
        let result = extraction_capabilities(&line);
        assert!(result.is_ok());
        let (hash, capabilities) = result.unwrap();
        assert_eq!(hash, "74730d410fcb6603ace96f1dc55ea6196122532d");
        assert_eq!(capabilities, vec!["capability1", "capability2", "capability3"]);
    }

    #[test]
    fn test_extraction_capabilities_empty() {
        let line = b"want 74730d410fcb6603ace96f1dc55ea6196122532d\n".to_vec();
        let result = extraction_capabilities(&line);
        println!("result: {:?}", result);
        assert!(result.is_ok());
        let (hash, capabilities) = result.unwrap();
        assert_eq!(hash, "74730d410fcb6603ace96f1dc55ea6196122532d");
        assert_eq!(capabilities.len(), 0);
    }

    #[test]
    fn test_receive_request_valid() {

        let mut lines = Vec::new();
        lines.push(b"want 74730d410fcb6603ace96f1dc55ea6196122532d multi_ack side-band-64k ofs-delta".to_vec());
        lines.push(b"want 7d1665144a3a975c05f1f43902ddaf084e784dbe".to_vec());
        lines.push(b"want 5a3f6be755bbb7deae50065988cbfa1ffa9ab68a".to_vec());
        
        let result = process_received_requests(lines);
        assert!(result.is_ok());
        let (capabilities, request) = result.unwrap();

        assert_eq!(capabilities, vec!["multi_ack", "side-band-64k", "ofs-delta"]);
        assert_eq!(request, vec!["74730d410fcb6603ace96f1dc55ea6196122532d", "7d1665144a3a975c05f1f43902ddaf084e784dbe", "5a3f6be755bbb7deae50065988cbfa1ffa9ab68a"]);
    }

    #[test]
    fn test_receive_request_valid_capabilities_empty() {

        let mut lines = Vec::new();
        lines.push(b"want 74730d410fcb6603ace96f1dc55ea6196122532d".to_vec());
        lines.push(b"want 7d1665144a3a975c05f1f43902ddaf084e784dbe".to_vec());
        lines.push(b"want 5a3f6be755bbb7deae50065988cbfa1ffa9ab68a".to_vec());
        
        let result = process_received_requests(lines);
        println!("{:?}", result);
        assert!(result.is_ok());
        let (capabilities, request) = result.unwrap();

        assert_eq!(capabilities.len(), 0);
        assert_eq!(request, vec!["74730d410fcb6603ace96f1dc55ea6196122532d", "7d1665144a3a975c05f1f43902ddaf084e784dbe", "5a3f6be755bbb7deae50065988cbfa1ffa9ab68a"]);
    }
}