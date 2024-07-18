use std::io::Read;
use crate::{consts::PR_FOLDER, servers::errors::ServerError, util::files::create_directory};

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

/// Crea una carpeta de pull request (PR) dentro del directorio fuente especificado.
///
/// Esta función construye la ruta a la carpeta PR utilizando la ruta del directorio fuente proporcionada
/// y el nombre predefinido `PR_FOLDER`. Luego, intenta crear el directorio en la ruta construida.
///
/// # Parámetros
/// - `src`: Una referencia de cadena que representa la ruta al directorio fuente donde se debe crear la carpeta PR.
///
/// # Retornos
/// - `Result<(), ServerError>`: Devuelve `Ok(())` si el directorio se crea correctamente,
///   de lo contrario, devuelve un `Err(ServerError::CreatePrFolderError)` indicando un fallo al crear el directorio.
///
/// # Errores
/// - `ServerError::CreatePrFolderError`: Este error se devuelve si la creación del directorio falla.
///
pub fn create_pr_folder(src: &str) -> Result<(), ServerError>{
    let pr_folder_path = format!("{}/{}", src, PR_FOLDER);
    let pr_folder_path = std::path::Path::new(&pr_folder_path);
    match create_directory(&pr_folder_path) 
    {
        Ok(_) => Ok(()),
        Err(_) => Err(ServerError::CreatePrFolderError),
    }
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
