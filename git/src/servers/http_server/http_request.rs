use std::{collections::HashMap, sync::{mpsc::Sender, Arc, Mutex}};
use crate::{consts::{APPLICATION_JSON, APPLICATION_SERVER, CONTENT_LENGTH, CONTENT_TYPE, HTPP_SIGNATURE, HTTP_VERSION}, servers::errors::ServerError, util::logger::log_message_with_signature};
use super::{http_body::HttpBody, method::Method, status_code::StatusCode, utils::read_request};

/// Representa una solicitud HTTP.
///
/// Esta estructura contiene los datos principales de una solicitud HTTP, como el método,
/// la ruta y el cuerpo de la solicitud.
/// 
#[derive(Debug, PartialEq)]
pub struct HttpRequest {
    method: String,
    path: String,
    body: HttpBody,
    headers: HashMap<String, String>,
}

impl HttpRequest {
    /// Crea una nueva instancia de `HttpRequest`.
    ///
    /// # Argumentos
    ///
    /// * `method` - El método HTTP.
    /// * `path` - La ruta del recurso solicitado.
    /// * `body` - El cuerpo de la solicitud, representado como un objeto JSON.
    ///
    /// # Retorna
    ///
    /// Retorna una nueva instancia de `HttpRequest`.
    /// 
    pub fn new(method: String, path: String, body: HttpBody, headers: HashMap<String, String>) -> Self {
        HttpRequest { method, path, body , headers}
    }

    /// Crea una nueva instancia de `HttpRequest` a partir de un lector.
    ///
    /// # Argumentos
    ///
    /// * `reader` - Un lector que implementa el trait `Read`.
    ///
    /// # Errores
    ///
    /// Retorna un `StatusCode` si ocurre un error al leer la solicitud, informando el error.
    ///
    /// # Retorna
    ///
    /// Retorna una nueva instancia de `HttpRequest`.
    /// 
    pub fn new_from_reader(reader: &mut dyn std::io::Read) -> Result<Self, StatusCode> {
        let request = match read_request(reader){
            Ok(request) => request,
            Err(_) => return Err(StatusCode::BadRequest(ServerError::ReadRequest.to_string())),
        };
        parse_http_request(&request)
    }

    /// Maneja la solicitud HTTP y ejecuta la acción correspondiente.
    ///
    /// # Argumentos
    ///
    /// * `tx` - Un transmisor sincronizado para enviar mensajes.
    /// * source - Una referencia a la cadena que contiene el directorio fuente.
    /// * `signature` - La firma del cliente.
    ///
    /// # Errores
    ///
    /// Retorna un `ServerError` si el método HTTP no es soportado o si ocurre un error al manejar la solicitud.
    ///
    /// # Retorna
    ///
    /// Retorna un `Result` que contiene la respuesta en caso de éxito, o un `ServerError` en caso de error.
    pub fn handle_http_request(&self, source: &String, tx: &Arc<Mutex<Sender<String>>>, _signature: &String) -> Result<StatusCode, ServerError> {
        // Manejar la solicitud HTTP        
        let message = format!("{} request to path: {}", self.method, self.path);
        log_message_with_signature(&tx, HTPP_SIGNATURE, &message);

        let method = match Method::create_method(&self.method) {
            Ok(method) => method,
            Err(_) => return Ok(StatusCode::MethodNotAllowed),
        };

        method.handle_method(&self.path, &self.body, source, tx)
    }

    /// Obtiene la ruta de la solicitud HTTP.
    ///
    /// # Retornos
    /// 
    /// Devuelve una referencia a la ruta de la solicitud.
    pub fn get_path(&self) -> &str {
        &self.path
    }

    pub fn get_content_type(&self) -> String {
        self.headers.get(CONTENT_TYPE).unwrap_or(&APPLICATION_SERVER.to_string()).to_string()
    }

}


/// Parsea una solicitud HTTP en una instancia de `HttpRequest`.
///
/// Esta función toma una cadena que representa una solicitud HTTP completa y la analiza para
/// extraer el método HTTP, la ruta y el cuerpo de la solicitud. Si la solicitud no es válida,
/// retorna un error `ServerError`.
///
/// # Argumentos
///
/// * `request` - Una cadena que representa la solicitud HTTP completa.
///
/// # Errores
///
/// Retorna `ServerError` en los siguientes casos:
/// - Si la solicitud no contiene suficientes líneas para ser válida.
/// - Si la línea de solicitud no tiene el formato esperado.
/// - Si el cuerpo de la solicitud no es un JSON válido.
///
/// # Retorna
///
/// Retorna un `Result` que contiene una instancia de `HttpRequest` en caso de éxito, o un `StatusCode`
/// en caso de error.
///
fn parse_http_request(request: &str) -> Result<HttpRequest, StatusCode> {
    let lines: Vec<&str> = request.lines().collect();
    if lines.len() < 1 {
        return Err(StatusCode::BadRequest(ServerError::MissingRequestLine.to_string()));
    }

    // Parsear la línea de solicitud (GET /path HTTP/1.1)
    let (method, path, http_version) = parse_request_line(lines[0])?;
    if http_version != HTTP_VERSION {
        return Err(StatusCode::HttpVersionNotSupported);
    }

    // Parsear los encabezados
    let header_end_index = lines.iter().position(|&line| line.is_empty()).unwrap_or(lines.len());
    let headers = parse_headers(&lines[1..header_end_index]);

    // Obtener el cuerpo de la solicitud
    let body = parse_body(request, &headers)?;

    Ok(HttpRequest::new(method, path, body, headers))
}

/// Parsea el cuerpo de una solicitud HTTP basado en el tipo de contenido especificado en los encabezados.
///
/// Esta función toma una solicitud HTTP en forma de string y un `HashMap` de encabezados,
/// y devuelve un `Result` que contiene un `HttpBody` en caso de éxito, o un `ServerError`
/// en caso de error.
///
/// # Argumentos
///
/// * `request` - Una referencia a un string slice que contiene la solicitud HTTP completa.
/// * `headers` - Un `HashMap` que contiene los encabezados de la solicitud HTTP.
///
/// # Retornos
///
/// Esta función devuelve un `Result<HttpBody, ServerError>`. Si el cuerpo de la solicitud
/// se parsea correctamente, devuelve un `HttpBody` correspondiente al tipo de contenido.
/// Si ocurre un error durante el parseo, devuelve un `ServerError`.
///
/// # Errores
///
/// Esta función puede devolver un `ServerError` si:
/// - El tipo de contenido especificado no es soportado.
/// - Ocurre un error durante el parseo del cuerpo de la solicitud.
///
fn parse_body(request: &str, headers: &HashMap<String, String>) -> Result<HttpBody, StatusCode> {
    // Obtener el cuerpo de la solicitud
    let finish = headers.get(CONTENT_LENGTH).map(|v| v.parse::<usize>().unwrap_or(0)).unwrap_or(0);
    let body = &request[request.len() - finish..];

    // Parsear el cuerpo de la solicitud
    let binding = APPLICATION_JSON.to_string();
    let content_type = headers.get(CONTENT_TYPE).unwrap_or(&binding);
    match HttpBody::parse(content_type, &body){
        Ok(body) => Ok(body),
        Err(_) => Err(StatusCode::UnsupportedMediaType),
    }

}

/// Analiza la línea de solicitud de una solicitud HTTP.
///
/// La línea de solicitud es la primera línea en una solicitud HTTP, típicamente en el formato:
/// `MÉTODO /ruta HTTP/versión`.
///
/// # Argumentos
///
/// * `line` - Una porción de cadena que contiene la línea de solicitud.
///
/// # Retornos
///
/// Si la línea de solicitud está correctamente formateada, devuelve una tupla que contiene el método, la ruta y la versión HTTP como cadenas.
/// De lo contrario, devuelve un error `ServerError::IncompleteRequestLine`.
///
/// # Errores
///
/// Devuelve `ServerError::IncompleteRequestLine` si la línea de solicitud no contiene al menos tres partes.
/// 
fn parse_request_line(line: &str) -> Result<(String, String, String), StatusCode> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 3 {
        return Err(StatusCode::BadRequest(ServerError::IncompleteRequestLine.to_string()));
    }
    Ok((parts[0].to_string(), parts[1].to_string(), parts[2].to_string()))
}

/// Analiza los encabezados de una solicitud HTTP.
///
/// # Argumentos
///
/// * `lines` - Un vector de porciones de cadena que contiene las líneas de los encabezados.
///
/// # Retornos
///
/// Un `HashMap` que mapea los nombres de los encabezados a sus valores.
/// 
fn parse_headers(lines: &[&str]) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    for line in lines {
        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim().to_string();
            let value = line[(colon_pos + 1)..].trim().to_string();
            headers.insert(key, value);
        }
    }
    headers
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_valid_request() {
        let request_str = "POST /path HTTP/1.1\r\nContent-Length: 18\r\n\r\n{\"key\": \"value\"}";
        let expected_request = HttpRequest {
            method: "POST".to_string(),
            path: "/path".to_string(),
            body: HttpBody::Json(json!({"key": "value"})),
            headers: [("Content-Length", "18")].iter().cloned().map(|(a, b)| (a.to_string(), b.to_string())).collect(),
        };
        assert_eq!(parse_http_request(request_str).unwrap(), expected_request);
    }

    #[test]
    fn test_parse_empty_request() {
        let request_str = "";
        assert!(parse_http_request(request_str).is_err());
    }

    #[test]
    fn test_parse_invalid_request_line() {
        let request_str = "GET";
        assert!(parse_http_request(request_str).is_err());
    }

    #[test]
    fn test_parse_invalid_json_body() {
        let request_str = "POST /path HTTP/1.1\r\nContent-Length: 18\r\n\r\n{\"key\": \"value\"";
        assert!(parse_http_request(request_str).is_err());
    }
}