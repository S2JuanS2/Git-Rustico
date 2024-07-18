use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use crate::errors::GitError;
// use crate::servers::errors::ServerError;
use super::http_request::HttpRequest;
use super::pr::PullRequest;


pub fn handle_client_http(
    stream: &mut TcpStream,
    tx: Arc<Mutex<Sender<String>>>,
    root_directory: String,
) -> Result<(), GitError> {
    // Creo la solicitud HTTP
    let request = HttpRequest::new_from_reader(stream)?;
    // Manejar la solicitud HTTP
    let pr = match PullRequest::from_json(&request.body)
    {
        Ok(pr) => pr,
        Err(e) => {
            let message = format!("Error: {:?}", e);
            println!("{}", message);
            return Err(e.into());
        }
    };
    let _response = request.handle_http_request(&pr,&root_directory, &tx)?;
    // // Enviar la respuesta al cliente
    // send_response(stream, &response)?;
    
    Ok(())
}