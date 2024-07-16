use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
// use http_request::HttpRequest;

use crate::errors::GitError;
use crate::servers::errors::ServerError;
use crate::servers::http_server::utils::read_request;

use super::http_request::HttpRequest;


pub fn handle_client_http(
    stream: &mut TcpStream,
    _tx: Arc<Mutex<Sender<String>>>,
    _root_directory: String,
) -> Result<(), GitError> {
    print!("HTTP");
    // Leer datos del cliente
    let request = read_request(stream)?;
    // // Parsear la solicitud HTTP
    let http_request = parse_http_request(&request)?;
    println!("Llegue");
    println!("{:?}", http_request);
    // // Manejar la solicitud HTTP
    // let response = handle_http_request(request, tx)?;
    
    // // Enviar la respuesta al cliente
    // send_response(stream, &response)?;
    
    Ok(())
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

    Ok(HttpRequest::new(method, path, body))
}