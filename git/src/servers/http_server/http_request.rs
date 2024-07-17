use std::sync::{mpsc::Sender, Arc, Mutex};

use serde_json::Value;

use crate::{servers::errors::ServerError, util::logger::log_message};

use super::utils::read_request;

#[derive(Debug)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub body: Value,
}

impl HttpRequest {
    pub fn new(method: String, path: String, body: Value) -> Self {
        HttpRequest { method, path, body }
    }

    pub fn new_from_reader(reader: &mut dyn std::io::Read) -> Result<Self, ServerError> {
        // Leer datos del cliente
        let request = read_request(reader)?;
        // Parsear la solicitud HTTP
        parse_http_request(&request)
    }

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