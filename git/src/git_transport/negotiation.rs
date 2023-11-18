use crate::{
    consts::{PKT_DONE, PKT_NAK},
    git_server::GitServer,
    util::{
        connections::{received_message, send_flush, send_message},
        errors::UtilError,
        pkt_line,
        validation::is_valid_obj_id, objects,
    },
};
use std::{io::{Read, Write}, net::TcpStream};

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
pub fn upload_request(socket: &mut TcpStream, advertised: &GitServer) -> Result<(), UtilError> {
    for refs in advertised.get_references() {
        let message = format!("want {}\n", refs.get_hash());
        let message = pkt_line::add_length_prefix(&message, message.len());
        send_message(socket, &message, UtilError::UploadRequest)?;
    }
    send_flush(socket, UtilError::UploadRequestFlush)?;
    Ok(())
}

/// Recibe y procesa un mensaje de no confirmación (NAK) del flujo de entrada.
///
/// Lee del flujo `stream` un mensaje de no confirmación (NAK) con un formato esperado.
///
/// # Argumentos
///
/// * `stream` - Una referencia mutable al flujo de entrada (implementa `Read`).
///
/// # Errores
///
/// Puede devolver un error en los siguientes casos:
///
/// - Si hay un error al leer los bytes del flujo de entrada o si la lectura no coincide con el mensaje NAK,
///   se devuelve un error `PackfileNegotiationReceiveNAK`.
///
/// # Retorno
///
/// Devuelve un `Result` que contiene `()` en caso de éxito o un error (`UtilError`) si falla la lectura
/// del mensaje NAK o si el mensaje recibido no coincide con el esperado.
///
pub fn receive_nak(stream: &mut dyn Read) -> Result<(), UtilError> {
    let mut buffer = [0u8; 8]; // Tamaño suficiente para "0008NAK\n"
    if stream.read_exact(&mut buffer).is_err() {
        return Err(UtilError::PackfileNegotiationReceiveNAK);
    }
    let response = String::from_utf8_lossy(&buffer);

    if response != PKT_NAK {
        return Err(UtilError::PackfileNegotiationReceiveNAK);
    }
    Ok(())
}

/// Recibe y verifica el mensaje de finalización "done" del cliente.
///
/// Esta función se utiliza para recibir el mensaje de finalización "done" del cliente
/// en el flujo de entrada proporcionado. Verifica que el mensaje recibido sea "done",
/// y en caso de éxito, retorna `Ok(())`. En caso de error o si el mensaje no es "done",
/// retorna un `Err(UtilError)`.
///
/// # Argumentos
///
/// * `stream` - Una referencia mutable a un objeto que implementa el trait `Read`,
///              que representa el flujo de entrada desde el cliente.
/// * `err`    - Un error específico que se retornará en caso de que la verificación falle.
///
/// # Retorna
///
/// Retorna un `Result` indicando el éxito (`Ok(())`) o un error (`Err(UtilError)`).
///
pub fn receive_done(stream: &mut dyn Read, err: UtilError) -> Result<(), UtilError> {
    let mut buffer = [0u8; 0]; // Tamaño suficiente para "0009done\n"
    if stream.read_exact(&mut buffer).is_err() {
        return Err(err);
    }
    let response = String::from_utf8_lossy(&buffer);

    if response != PKT_DONE {
        return Err(err);
    }
    Ok(())
}

/// Recibe una solicitud del cliente durante la fase de negociación del protocolo de transporte de Git.
/// La solicitud puede incluir mensajes "want", "have" y "done".
///
/// # Argumentos
///
/// * `stream` - Una referencia mutable a un objeto `Read` que representa el flujo de entrada desde el cliente.
///
/// # Retorna
///
/// Un `Result` que contiene una tupla con tres vectores: `capacities`, `wanted_objects` y `common_objects`.
/// - `capacities`: Un vector de cadenas que representa las capacidades solicitadas por el cliente.
/// - `wanted_objects`: Un vector de cadenas que representa los objetos solicitados por el cliente mediante "want".
/// - `common_objects`: Un vector de cadenas que representa los objetos que el cliente ya tiene mediante "have".
///
/// # Errores
///
/// Retorna un `UtilError` en caso de cualquier problema durante la comunicación o si no se recibe el mensaje "done" esperado.

pub fn receive_request(
    stream: &mut dyn Read,
) -> Result<(Vec<String>, Vec<String>, Vec<String>), UtilError> {
    // Want
    let lines = pkt_line::read(stream)?;
    let (capacilities, request) = process_received_requests_want(lines)?;

    let lines = pkt_line::read(stream)?;

    if lines.len() == 1 && lines[0] == b"done" {
        return Ok((capacilities, request, Vec::new()));
    }

    // Have
    let request_have = receive_request_type(lines, "have", UtilError::UnexpectedRequestNotHave)?;

    // Done
    received_message(stream, PKT_DONE, UtilError::NegociacionExpectedDone)?;
    Ok((capacilities, request, request_have))
}

/// Procesa las solicitudes recibidas a partir de un conjunto de líneas de bytes.
///
/// Toma un vector de vectores de bytes `lines` que representan las solicitudes recibidas.
/// Itera sobre las líneas y extrae las solicitudes de tipo "want" y sus capacidades.
///
/// # Argumentos
///
/// * `lines` - Vector de vectores de bytes que contiene las líneas con las solicitudes recibidas.
///
/// # Errores
///
/// Puede devolver un error en los siguientes casos:
///
/// - Si hay un error al extraer las capacidades o el hash, se devuelve el error correspondiente.
/// - Si hay un error al procesar las solicitudes, se devuelve el error específico asociado.
///
///
/// # Retorno
///
/// Devuelve un `Result` que contiene una tupla con un vector de capacidades en formato de cadenas
/// y un vector de hashes de solicitudes de tipo "want" en formato de cadenas (`(Vec<String>, Vec<String>)`),
/// o un error (`UtilError`) en caso de que falle el procesamiento de las solicitudes recibidas.
///
fn process_received_requests_want(
    lines: Vec<Vec<u8>>,
) -> Result<(Vec<String>, Vec<String>), UtilError> {
    let mut request = Vec::new();

    // Want and capabilities
    let (hash, capacilities) = extraction_capabilities(&lines[0])?;
    request.push(hash);

    // Want
    let want = receive_request_type(
        lines[1..].to_vec(),
        "want",
        UtilError::UnexpectedRequestNotWant,
    )?;
    request.extend(want);

    Ok((capacilities, request))
}

/// Extrae las capacidades y el hash de una línea de bytes.
///
/// Toma una referencia a un vector de bytes `line` y busca las capacidades y el hash
/// de una solicitud determinada.
///
/// # Argumentos
///
/// * `line` - Referencia al vector de bytes que contiene la línea con las capacidades y el hash.
///
/// # Errores
///
/// Puede devolver un error en los siguientes casos:
///
/// - Si no se encuentra el tipo de solicitud esperado, se devuelve `UtilError::UnexpectedRequestNotWant`.
/// - Si hay un error en el formato de la solicitud, se devuelve `UtilError::InvalidRequestFormat`.
/// - Si el identificador de objeto (hash) no es válido, se devuelve `UtilError::InvalidObjectId`.
///
/// # Retorno
///
/// Devuelve un `Result` que contiene una tupla con el hash en formato de cadena y un vector de capacidades
/// en formato de cadenas (`(String, Vec<String>)`), o un error (`UtilError`) en caso de que falle la extracción
/// o validación de las capacidades y el hash.
///
fn extraction_capabilities(line: &[u8]) -> Result<(String, Vec<String>), UtilError> {
    let line_str = String::from_utf8_lossy(line);
    let mut line_split = line_str.split_ascii_whitespace();
    let type_request = line_split
        .next()
        .ok_or_else(|| UtilError::InvalidRequestFormat(line_str.to_string()))?;
    if type_request != "want" {
        return Err(UtilError::UnexpectedRequestNotWant);
    }
    let hash = line_split
        .next()
        .ok_or_else(|| UtilError::InvalidRequestFormat(line_str.to_string()))?;
    if !is_valid_obj_id(hash) {
        return Err(UtilError::InvalidObjectId);
    }
    let capacilities = line_split
        .collect::<Vec<&str>>()
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    Ok((hash.to_string(), capacilities))
}

/// Extrae y procesa las solicitudes de un tipo específico a partir de un conjunto de líneas.
///
/// Toma un vector de vectores de bytes `lines` y un `type_req` que indica el tipo de solicitud esperado.
/// Itera sobre las líneas y extrae las solicitudes del tipo requerido, validando y obteniendo los hashes.
///
/// # Argumentos
///
/// * `lines` - Vector que contiene las líneas de bytes con las solicitudes.
/// * `type_req` - Tipo de solicitud esperada, por ejemplo, "want".
/// * `err` - Error específico a devolver en caso de que no se encuentre el tipo de solicitud esperado.
///
/// # Errores
///
/// Puede devolver un error en los siguientes casos:
///
/// - Si la solicitud no comienza con el tipo requerido, se devuelve `UtilError::UnexpectedRequestNotWant`.
/// - Si hay un error en el formato de la solicitud, se devuelve `UtilError::InvalidRequestFormat`.
/// - Si el identificador de objeto (hash) no es válido, se devuelve `UtilError::InvalidObjectId`.
///
/// # Retorno
///
/// Devuelve un `Result` que contiene un vector de cadenas (`Vec<String>`) con los hashes extraídos,
/// o un error (`UtilError`) en caso de que falle la extracción o validación de las solicitudes.
///
fn receive_request_type(
    lines: Vec<Vec<u8>>,
    type_req: &str,
    error: UtilError,
) -> Result<Vec<String>, UtilError> {
    lines.iter().try_fold(Vec::new(), |mut acc, line| {
        let line_str = String::from_utf8_lossy(line);

        if !line_str.starts_with(type_req) {
            return Err(error.clone());
        }

        let request = line_str.trim().to_string();
        let hash = request
            .split_ascii_whitespace()
            .nth(1)
            .ok_or_else(|| UtilError::InvalidRequestFormat(request.to_string()))?;

        if !is_valid_obj_id(hash) {
            return Err(UtilError::InvalidObjectId);
        }

        acc.push(hash.to_string());
        Ok(acc)
    })
}

/// Envia las referencias confirmadas al cliente durante la fase de negociación.
///
/// Para cada identificador de objeto en el vector proporcionado, esta función envía un
/// mensaje de agradecimiento al cliente, indicando que el objeto es reconocido y que la
/// transferencia puede continuar. Después de procesar todos los objetos, envía un mensaje
/// de agradecimiento negativo (NAK) para confirmar las referencias y señalar el final de
/// la fase de agradecimiento.
///
/// # Argumentos
///
/// * `stream` - Una referencia mutable a un objeto de tipo trait que implementa el trait
///              `Write`, que representa el flujo de salida hacia el cliente.
/// * `objt_id` - Un vector que contiene los identificadores de objetos para los cuales
///               se deben enviar mensajes de agradecimiento.
///
/// # Retorna
///
/// Retorna un `Result` indicando el éxito (`Ok(())`) o un error (`Err(UtilError)`).
///
pub fn sent_references_valid_client(stream: &mut dyn Write, objt_id: &Vec<String>) -> Result<(), UtilError>
{
    for obj in objt_id {
        let message = format!("ACK{} continue\n", obj);
        let message = pkt_line::add_length_prefix(&message, message.len());
        send_message(stream, &message, UtilError::UploadRequest)?;
    }
    send_message(stream, PKT_NAK, UtilError::SendNAKConfirmReferences)?; // SendNAKConfirmReferences
    Ok(())
}

/// Envia una confirmacion ACK por la última referencia al cliente, para cerrar la etapa 
/// çde negociacion.
///
/// Esta función envía un mensaje de agradecimiento al cliente por el último objeto en
/// el vector proporcionado. El mensaje de agradecimiento indica que el objeto es reconocido
/// y la transferencia puede continuar.
///
/// # Argumentos
///
/// * `writer` - Una referencia mutable a un objeto que implementa el trait `Write`, que
///              representa el flujo de salida hacia el cliente.
/// * `objs`   - Un vector que contiene los identificadores de objetos, y se asume que
///              está ordenado de manera que el último objeto en el vector es el más reciente.
///
/// # Retorna
///
/// Retorna un `Result` indicando el éxito (`Ok(())`) o un error (`Err(UtilError)`).
///
pub fn send_acknowledge_last_reference(writer: &mut dyn Write, objs: &Vec<String>) -> Result<(), UtilError>
{
    let message = format!("ACK {}\n", objs[objs.len() - 1]);
    send_message(writer, &message, UtilError::SendLastACKConf)
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
        let result = receive_request_type(lines, "want", UtilError::UnexpectedRequestNotWant);
        assert!(result.is_ok());
        let wanted_hashes = result.unwrap();
        assert_eq!(
            wanted_hashes,
            vec![
                "74730d410fcb6603ace96f1dc55ea6196122532d",
                "7d1665144a3a975c05f1f43902ddaf084e784dbe",
                "5a3f6be755bbb7deae50065988cbfa1ffa9ab68a"
            ]
        );
    }

    #[test]
    fn test_receive_request_type_valid_have() {
        let mut lines = Vec::new();
        lines.push(b"have 7e47fe2bd8d01d481f44d7af0531bd93d3b21c01".to_vec());
        lines.push(b"have 74730d410fcb6603ace96f1dc55ea6196122532d".to_vec());
        let result = receive_request_type(lines, "have", UtilError::UnexpectedRequestNotHave);
        assert!(result.is_ok());
        let have_hashes = result.unwrap();
        assert_eq!(
            have_hashes,
            vec![
                "7e47fe2bd8d01d481f44d7af0531bd93d3b21c01",
                "74730d410fcb6603ace96f1dc55ea6196122532d"
            ]
        );
    }

    #[test]
    fn test_receive_request_type_invalid() {
        let mut lines = Vec::new();
        lines.push(b"have 74730d410fcb6603ace96f1dc55ea6196122532d".to_vec());
        lines.push(b"want 7d1665144a3a975c05f1f43902ddaf084e784dbe".to_vec());
        lines.push(b"have 5a3f6be755bbb7deae50065988cbfa1ffa9ab68a".to_vec());
        let result = receive_request_type(lines, "want", UtilError::UnexpectedRequestNotWant);
        assert!(result.is_err());
    }

    #[test]
    fn test_receive_request_empty() {
        let lines = Vec::new();
        let result = receive_request_type(lines, "want", UtilError::UnexpectedRequestNotWant);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_extraction_capabilities_valid() {
        let line =
            b"want 74730d410fcb6603ace96f1dc55ea6196122532d capability1 capability2 capability3\n"
                .to_vec();
        let result = extraction_capabilities(&line);
        assert!(result.is_ok());
        let (hash, capabilities) = result.unwrap();
        assert_eq!(hash, "74730d410fcb6603ace96f1dc55ea6196122532d");
        assert_eq!(
            capabilities,
            vec!["capability1", "capability2", "capability3"]
        );
    }

    #[test]
    fn test_extraction_capabilities_empty() {
        let line = b"want 74730d410fcb6603ace96f1dc55ea6196122532d\n".to_vec();
        let result = extraction_capabilities(&line);
        // println!("result: {:?}", result);
        assert!(result.is_ok());
        let (hash, capabilities) = result.unwrap();
        assert_eq!(hash, "74730d410fcb6603ace96f1dc55ea6196122532d");
        assert_eq!(capabilities.len(), 0);
    }

    #[test]
    fn test_receive_request_valid() {
        let mut lines = Vec::new();
        lines.push(
            b"want 74730d410fcb6603ace96f1dc55ea6196122532d multi_ack side-band-64k ofs-delta"
                .to_vec(),
        );
        lines.push(b"want 7d1665144a3a975c05f1f43902ddaf084e784dbe".to_vec());
        lines.push(b"want 5a3f6be755bbb7deae50065988cbfa1ffa9ab68a".to_vec());

        let result = process_received_requests_want(lines);
        assert!(result.is_ok());
        let (capabilities, request) = result.unwrap();

        assert_eq!(
            capabilities,
            vec!["multi_ack", "side-band-64k", "ofs-delta"]
        );
        assert_eq!(
            request,
            vec![
                "74730d410fcb6603ace96f1dc55ea6196122532d",
                "7d1665144a3a975c05f1f43902ddaf084e784dbe",
                "5a3f6be755bbb7deae50065988cbfa1ffa9ab68a"
            ]
        );
    }

    #[test]
    fn test_receive_request_valid_capabilities_empty() {
        let mut lines = Vec::new();
        lines.push(b"want 74730d410fcb6603ace96f1dc55ea6196122532d".to_vec());
        lines.push(b"want 7d1665144a3a975c05f1f43902ddaf084e784dbe".to_vec());
        lines.push(b"want 5a3f6be755bbb7deae50065988cbfa1ffa9ab68a".to_vec());

        let result = process_received_requests_want(lines);
        println!("{:?}", result);
        assert!(result.is_ok());
        let (capabilities, request) = result.unwrap();

        assert_eq!(capabilities.len(), 0);
        assert_eq!(
            request,
            vec![
                "74730d410fcb6603ace96f1dc55ea6196122532d",
                "7d1665144a3a975c05f1f43902ddaf084e784dbe",
                "5a3f6be755bbb7deae50065988cbfa1ffa9ab68a"
            ]
        );
    }
}
