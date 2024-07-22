use std::{collections::HashMap, sync::{mpsc::Sender, Arc, Mutex}};
use crate::{consts::HTTP_VERSION, servers::errors::ServerError};
use super::{http_body::HttpBody, pr::PullRequest, status_code::StatusCode, utils::read_request};

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
    /// Retorna un `ServerError` si ocurre un error al leer o parsear la solicitud.
    ///
    /// # Retorna
    ///
    /// Retorna una nueva instancia de `HttpRequest`.
    /// 
    pub fn new_from_reader(reader: &mut dyn std::io::Read) -> Result<Self, ServerError> {
        let request = read_request(reader)?;
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
        let pr = PullRequest::from_http_body(&self.body)?;
        println!("Pull Request: {:?}", pr);
        match self.method.as_str() {
            "GET" => self.handle_get_request(&pr, source, tx),
            "POST" => self.handle_post_request(&pr, source, tx),
            "PUT" => self.handle_put_request(&pr, source, tx),
            "PATCH" => self.handle_patch_request(&pr, source, tx),
            _ => Err(ServerError::MethodNotAllowed),
        }
    }

    /// Obtiene la ruta de la solicitud HTTP.
    ///
    /// # Retornos
    /// 
    /// Devuelve una referencia a la ruta de la solicitud.
    pub fn get_path(&self) -> &str {
        &self.path
    }

     /// Maneja una solicitud HTTP GET.
    ///
    /// # Parámetros
    /// 
    /// * `pr` - Una referencia a la estructura `PullRequest`.
    /// * `src` - Una referencia a la cadena que contiene el directorio fuente.
    /// * `tx` - Un puntero compartido y seguro para subprocesos a un transmisor de mensajes.
    ///
    /// # Retornos
    /// 
    /// Devuelve un `Result` que contiene la respuesta en caso de éxito o un `ServerError` en caso de fallo.
    /// 
    fn handle_get_request(&self, pr: &PullRequest, src: &String, tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
        let path_segments: Vec<&str> = self.get_path().split('/').collect();
        match path_segments.as_slice() {
            ["repos", repo_name, "pulls"] => {
                return pr.list_pull_request(repo_name, src, tx);
            },
            ["repos", repo_name, "pulls", pull_number] => {
                return pr.get_pull_request(repo_name, pull_number, src, tx);
            },
            ["repos", repo_name, "pulls", pull_number, "commits"] => {
                return pr.list_commits(repo_name, pull_number, src, tx);
            },
            _ => {
                return Err(ServerError::InvalidGetPathError);
            }
        }
    }
    
    /// Maneja una solicitud HTTP POST.
    ///
    /// # Parámetros
    /// 
    /// * `pr` - Una referencia a la estructura `PullRequest`.
    /// * `src` - Una referencia a la cadena que contiene el directorio fuente.
    /// * `tx` - Un puntero compartido y seguro para subprocesos a un transmisor de mensajes.
    ///
    /// # Retornos
    /// 
    /// Devuelve un `Result` que contiene la respuesta en caso de éxito o un `ServerError` en caso de fallo.
    /// 
    fn handle_post_request(&self, pr: &PullRequest, src: &String, tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
        let path_segments: Vec<&str> = segment_path(&self.get_path());
        match path_segments.as_slice() {
            ["repos", repo_name, "pulls"] => {
                return pr.create_pull_requests(repo_name, src, tx);
            }
            _ => {
                return Err(ServerError::InvalidPostPathError);
            }
        }
    }
    
     /// Maneja una solicitud HTTP PUT.
    ///
    /// # Parámetros
    /// 
    /// * `pr` - Una referencia a la estructura `PullRequest`.
    /// * `src` - Una referencia a la cadena que contiene el directorio fuente.
    /// * `tx` - Un puntero compartido y seguro para subprocesos a un transmisor de mensajes.
    ///
    /// # Retornos
    /// 
    /// Devuelve un `Result` que contiene la respuesta en caso de éxito o un `ServerError` en caso de fallo.
    /// 
    fn handle_put_request(&self, pr: &PullRequest, src: &String, tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
        let path_segments: Vec<&str> = segment_path(&self.get_path());
        match path_segments.as_slice() {
            ["repos", repo_name, "pulls", pull_number, "merge"] => {
                return pr.merge_pull_request(repo_name, pull_number, src, tx);
            },
            _ => {
                return Err(ServerError::InvalidPutPathError);
            }
        }
    }
    
    /// Maneja una solicitud HTTP PATCH.
    ///
    /// # Parámetros
    /// 
    /// * `pr` - Una referencia a la estructura `PullRequest`.
    /// * `src` - Una referencia a la cadena que contiene el directorio fuente.
    /// * `tx` - Un puntero compartido y seguro para subprocesos a un transmisor de mensajes.
    ///
    /// # Retornos
    /// 
    /// Devuelve un `Result` que contiene la respuesta en caso de éxito o un `ServerError` en caso de fallo.
    /// 
    fn handle_patch_request(&self, pr: &PullRequest, src: &String, tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
        let path_segments: Vec<&str> = segment_path(&self.get_path());
        match path_segments.as_slice() {
            ["repos", repo_name, "pulls", pull_number] => {
                return pr.modify_pull_request(repo_name, pull_number, src, tx);
            },
            _ => {
                Err(ServerError::InvalidPatchPathError)
            }
        }
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
/// Retorna un `Result` que contiene una instancia de `HttpRequest` en caso de éxito, o un `ServerError`
/// en caso de error.
///
fn parse_http_request(request: &str) -> Result<HttpRequest, ServerError> {
    let lines: Vec<&str> = request.lines().collect();
    if lines.len() < 1 {
        return Err(ServerError::MissingRequestLine);
    }

    // Parsear la línea de solicitud (GET /path HTTP/1.1)
    let (method, path, http_version) = parse_request_line(lines[0])?;
    if http_version != HTTP_VERSION {
        return Err(ServerError::HttpVersionNotSupported);
    }

    // Parsear los encabezados
    let header_end_index = lines.iter().position(|&line| line.is_empty()).unwrap_or(lines.len());
    let headers = parse_headers(&lines[1..header_end_index]);

    // Obtener el cuerpo de la solicitud
    let body = if header_end_index < lines.len() {
        lines[(header_end_index + 1)..].join("\n")
    } else {
        String::new()
    };

    // Parsear el cuerpo de la solicitud
    let binding = "application/json".to_string();
    let content_type = headers.get("Content-Type").unwrap_or(&binding);
    let body = HttpBody::parse(content_type, &body)?;

    Ok(HttpRequest::new(method, path, body, headers))
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
fn parse_request_line(line: &str) -> Result<(String, String, String), ServerError> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 3 {
        return Err(ServerError::IncompleteRequestLine);
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
/// Segmenta una ruta en partes separadas.
///
/// # Argumentos
///
/// * `path` - Una cadena que contiene la ruta a segmentar.
///
/// # Retornos
///
/// Un vector de porciones de cadena que representan las partes segmentadas de la ruta.
/// 
pub fn segment_path(path: &str) -> Vec<&str> {
    // debo eliminar el 1ero si es solo un /
    let mut path = path;
    if path.starts_with("/") {
        path = &path[1..];
    }
    path.split('/').collect()
}

/// Analiza el cuerpo de una solicitud HTTP según el tipo de contenido.
///
/// # Argumentos
///
/// * `content_type` - Una cadena que indica el tipo de contenido del cuerpo.
/// * `body` - Una cadena que contiene el cuerpo de la solicitud.
///
/// # Retornos
///
/// Un valor de `serde_json::Value` si el cuerpo se puede analizar correctamente.
/// De lo contrario, devuelve un error `ServerError`.
///
/// # Errores
///
/// Devuelve `ServerError::HttpParseBody` si el cuerpo no se puede analizar como JSON.
/// Devuelve `ServerError::UnsupportedMediaType` si el tipo de contenido no es compatible.
/// 
// fn parse_body(content_type: &str, body: &str) -> Result<Value, ServerError> {
//     match content_type {
//         "application/json" => serde_json::from_str(body).map_err(|_| ServerError::HttpJsonParseBody),
//         "text/plain" => Ok(Value::String(body.to_string())),
//         "application/xml" => serde_xml_rs::from_str(body).map_err(|_| ServerError::HttpXmlParseBody),
//         "application/x-yaml" | "text/yaml" => serde_yaml::from_str(body).map_err(|_| ServerError::HttpYamlParseBody),
//         _ => Err(ServerError::UnsupportedMediaType),
//     }
// }

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