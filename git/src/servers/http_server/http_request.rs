use crate::servers::errors::ServerError;

use super::utils::read_request;


#[derive(Debug)]
pub struct HttpRequest {
    method: String,
    path: String,
    body: String,
}

impl HttpRequest {
    pub fn new(method: String, path: String, body: String) -> Self {
        HttpRequest { method, path, body }
    }

    pub fn new_from_reader(reader: &mut dyn std::io::Read) -> Result<Self, ServerError> {
        let content = read_request(reader)?;
        println!("{}", content);
        // pass
        return Ok(HttpRequest::new("GET".to_string(), "/".to_string(), "".to_string()));
    }

    pub fn print(&self) {
        println!("Method: {}", self.method);
        println!("Path: {}", self.path);
        println!("Body: {}", self.body);
    }
}