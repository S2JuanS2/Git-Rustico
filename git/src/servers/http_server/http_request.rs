use std::sync::{mpsc::Sender, Arc, Mutex};
use serde_json::Value;
use crate::{servers::errors::ServerError, util::logger::log_message};
use super::utils::read_request;

/// Representa una solicitud HTTP.
///
/// Esta estructura contiene los datos principales de una solicitud HTTP, como el método,
/// la ruta y el cuerpo de la solicitud.
/// 
#[derive(Debug)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub body: Value,
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
    pub fn handle_http_request(&self, tx: &Arc<Mutex<Sender<String>>>) -> Result<String, ServerError> {
        // Manejar la solicitud HTTP
        match self.method.as_str() {
            "GET" => self.handle_get_request(),
            "POST" => self.handle_post_request(tx),
            "PUT" => self.handle_put_request(tx),
            "PATCH" => self.handle_patch_request(tx),
            _ => Err(ServerError::MethodNotAllowed),
        }
    }

    fn handle_get_request(&self) -> Result<String, ServerError> {
        println!("GET request to path: {}", self.path);
        Ok(format!("GET request to path: {}", self.path))
    }

    fn handle_post_request(&self, tx: &Arc<Mutex<Sender<String>>>) -> Result<String, ServerError> {
        let message = self.body["message"].as_str().unwrap_or("No message");
        let message = message.to_string();
        log_message(&tx, &message);
        println!("POST request to path: {} with message: {}", self.path, message);
        Ok(format!("POST request to path: {} with message: {}", self.path, message))
    }

    fn handle_put_request(&self, tx: &Arc<Mutex<Sender<String>>>) -> Result<String, ServerError> {
        let message = self.body["message"].as_str().unwrap_or("No message");
        let message = message.to_string();
        log_message(&tx, &message);
        println!("PUT request to path: {} with message: {}", self.path, message);
        Ok(format!("PUT request to path: {} with message: {}", self.path, message))
    }

    fn handle_patch_request(&self, tx: &Arc<Mutex<Sender<String>>>) -> Result<String, ServerError> {
        let message = self.body["message"].as_str().unwrap_or("No message");
        let message = message.to_string();
        log_message(&tx, &message);
        println!("PATCH request to path: {} with message: {}", self.path, message);
        Ok(format!("PATCH request to path: {} with message: {}", self.path, message))
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
