use serde_json::Value;

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

    // pub fn new_from_reader(reader: &mut dyn std::io::Read) -> Result<Self, ServerError> {
    //     let content = read_request(reader)?;
    //     // pass
    //     return Ok(HttpRequest::new("GET".to_string(), "/".to_string(), "".to_string()));
    // }
}