use std::io::Read;

use crate::servers::errors::ServerError;


pub fn read_request(reader: &mut dyn Read) -> Result<String, ServerError> {
    let mut buffer = [0; 512];
    let mut request = Vec::new();

    loop {
        let bytes_read = match reader.read(&mut buffer)
        {
            Ok(bytes_read) => bytes_read,
            Err(_) => return Err(ServerError::ReadHttpRequest),
        };
        if bytes_read == 0 {
            break;
        }
        request.extend_from_slice(&buffer[..bytes_read]);
        if bytes_read < buffer.len() {
            break;
        }
    }
    Ok(String::from_utf8_lossy(&request).to_string())
}