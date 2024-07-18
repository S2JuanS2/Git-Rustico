use std::sync::{mpsc::Sender, Arc, Mutex};
use serde_json::Value;
use crate::{servers::errors::ServerError, util::logger::log_message};
use super::{pr::PullRequest, utils::read_request};

/// Representa una solicitud HTTP.
///
/// Esta estructura contiene los datos principales de una solicitud HTTP, como el método,
/// la ruta y el cuerpo de la solicitud.
/// 
#[derive(Debug, PartialEq)]
pub struct HttpRequest {
    method: String,
    path: String,
    body: Value,
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
    pub fn new(method: String, path: String, body: Value) -> Self {
        HttpRequest { method, path, body }
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
    ///
    /// # Errores
    ///
    /// Retorna un `ServerError` si el método HTTP no es soportado o si ocurre un error al manejar la solicitud.
    ///
    /// # Retorna
    ///
    /// Retorna un `Result` que contiene la respuesta en caso de éxito, o un `ServerError` en caso de error.
    pub fn handle_http_request(&self, source: &String,tx: &Arc<Mutex<Sender<String>>>) -> Result<String, ServerError> {
        // Manejar la solicitud HTTP
        let pr = self.build_pull_request_from_body(tx)?;
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
    fn handle_get_request(&self, pr: &PullRequest, src: &String, tx: &Arc<Mutex<Sender<String>>>) -> Result<String, ServerError> {
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
                return Err(ServerError::MethodNotAllowed);
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
    fn handle_post_request(&self, pr: &PullRequest, src: &String, tx: &Arc<Mutex<Sender<String>>>) -> Result<String, ServerError> {
        let path_segments: Vec<&str> = self.get_path().split('/').collect();
        match path_segments.as_slice() {
            ["repos", repo_name, "pulls"] => {
                return pr.create_pull_requests(repo_name, src, tx);
            }
            _ => {
                return Err(ServerError::MethodNotAllowed);
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
    fn handle_put_request(&self, pr: &PullRequest, src: &String, tx: &Arc<Mutex<Sender<String>>>) -> Result<String, ServerError> {
        let path_segments: Vec<&str> = self.get_path().split('/').collect();
        match path_segments.as_slice() {
            ["repos", repo_name, "pulls", pull_number, "merge"] => {
                return pr.merge_pull_request(repo_name, pull_number, src, tx);
            },
            _ => {
                return Err(ServerError::MethodNotAllowed);
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
    fn handle_patch_request(&self, pr: &PullRequest, src: &String, tx: &Arc<Mutex<Sender<String>>>) -> Result<String, ServerError> {
        let path_segments: Vec<&str> = self.get_path().split('/').collect();
        match path_segments.as_slice() {
            ["repos", repo_name, "pulls", pull_number] => {
                return pr.modify_pull_request(repo_name, pull_number, src, tx);
            },
            _ => {
                Err(ServerError::MethodNotAllowed)
            }
        }
    }

     /// Crea una estructura `PullRequest` desde el cuerpo de la solicitud HTTP.
    ///
    /// # Parámetros
    /// 
    /// * `tx` - Un puntero compartido y seguro para subprocesos a un transmisor de mensajes.
    ///
    /// # Retornos
    /// 
    /// Devuelve un `Result` que contiene la estructura `PullRequest` en caso de éxito o un `ServerError` en caso de fallo.
    /// 
    fn build_pull_request_from_body(&self, tx: &Arc<Mutex<Sender<String>>>) -> Result<PullRequest, ServerError> {
        match PullRequest::from_json(&self.body)
        {
            Ok(pr) => Ok(pr),
            Err(e) => {
                let message = format!("Error en la solicitud HTTP. Error: {}", e);
                log_message(&tx, &message);
                println!("{}", message);
                Err(ServerError::HttpParseBody)
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
        return Err(ServerError::ServerDebug);
    }

    // Parsear la línea de solicitud (GET /path HTTP/1.1)
    let request_line: Vec<&str> = lines[0].split_whitespace().collect();
    if request_line.len() < 3 {
        return Err(ServerError::ServerDebug);
    }

    let method = request_line[0].to_string();
    let path = request_line[1].to_string();

    // Extraer el cuerpo de la solicitud (si existe)
    let body = if let Some(index) = lines.iter().position(|&line| line.is_empty()) {
        lines[(index + 1)..].join("\n")
    } else {
        String::new()
    };
    let body: Value = match serde_json::from_str(&body) {
        Ok(body) => body,
        Err(_) => return Err(ServerError::HttpParseBody),
    };

    Ok(HttpRequest::new(method, path, body))
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
            body: json!({"key": "value"}),
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