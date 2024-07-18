use std::io::Read;
use crate::servers::errors::ServerError;

/// Reads an HTTP request from a reader, returning it as a String.
///
/// # Arguments
///
/// * `reader` - A mutable reference to a type implementing the `Read` trait.
///
/// # Returns
///
/// Returns a `Result`:
/// - `Ok(String)` if reading and converting to UTF-8 was successful.
/// - `Err(ServerError)` if there was an error while reading.
/// 
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Read};

    #[test]
    fn test_read_request_valid_data() {
        // Simulate valid HTTP request data
        let request_data = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
        let mut cursor = Cursor::new(request_data);

        // Call the read_request function
        match read_request(&mut cursor) {
            Ok(request) => {
                assert_eq!(request, "GET / HTTP/1.1\r\nHost: example.com\r\n\r\n");
            }
            Err(err) => {
                panic!("Expected Ok result, but got Err: {:?}", err);
            }
        }
    }

    #[test]
    fn test_read_request_empty_data() {
        // Simulate empty input data
        let request_data = b"";
        let mut cursor = Cursor::new(request_data);

        // Call the read_request function
        match read_request(&mut cursor) {
            Ok(request) => {
                assert_eq!(request, "");
            }
            Err(err) => {
                panic!("Expected Ok result, but got Err: {:?}", err);
            }
        }
    }

    #[test]
    fn test_read_request_error() {
        // Simulate a reader that always returns an error
        struct ErrorReader;
        impl Read for ErrorReader {
            fn read(&mut self, _: &mut [u8]) -> Result<usize, std::io::Error> {
                Err(std::io::Error::new(std::io::ErrorKind::Other, "Simulated error"))
            }
        }

        let mut error_reader = ErrorReader;

        // Call the read_request function with the error reader
        match read_request(&mut error_reader) {
            Ok(_) => {
                panic!("Expected Err result, but got Ok");
            }
            Err(err) => {
                assert_eq!(err, ServerError::ReadHttpRequest);
            }
        }
    }
}
