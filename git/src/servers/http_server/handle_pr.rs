use crate::{servers::errors::ServerError, util::logger::log_message};
use super::http_request::HttpRequest;
use std::sync::{mpsc::Sender, Arc, Mutex};


pub fn handle_get_request(request: &HttpRequest) -> Result<String, ServerError> {
    let message = format!("GET request to path: {}", request.path);
    println!("{}", message);
    Ok(message)
}

pub fn handle_post_request(request: &HttpRequest, tx: &Arc<Mutex<Sender<String>>>) -> Result<String, ServerError> {
    let message = request.body["message"].as_str().unwrap_or("No message");
    let message = message.to_string();
    let message = format!("POST request to path: {} with message: {}", request.path, message);
    log_message(&tx, &message);
    println!("{}", message);

    // Fijarme si existe la carpeta .pr en source
    // Si no existe, crearla

    // Fijarte si existe una carpeta en .pr con el nombre del repositorio en el servidor, 
    // sino lo hay crearlo

    // Fijarme si el pr existe en la carpeta del repositorio, sino existe crearlo

    // Crear un archivo con el nombre del pr en la carpeta del pr

    // Escribir el contenido del pr en el archivo
    Ok(message)
}

pub fn handle_put_request(request: &HttpRequest, tx: &Arc<Mutex<Sender<String>>>) -> Result<String, ServerError> {
    let message = request.body["message"].as_str().unwrap_or("No message");
    let message = message.to_string();
    let message = format!("PUT request to path: {} with message: {}", request.path, message);
    log_message(&tx, &message);
    println!("{}", message);
    Ok(message)
}

pub fn handle_patch_request(request: &HttpRequest, tx: &Arc<Mutex<Sender<String>>>) -> Result<String, ServerError> {
    let message = request.body["message"].as_str().unwrap_or("No message");
    let message = message.to_string();
    let message = format!("PATCH request to path: {} with message: {}", request.path, message);
    log_message(&tx, &message);
    println!("{}", message);
    Ok(message)
}