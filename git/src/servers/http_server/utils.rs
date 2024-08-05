use std::{fs::OpenOptions, io::{Read, Seek, SeekFrom, Write}, num::ParseIntError, path::Path};
use crate::{consts::{APPLICATION_SERVER, CRLF, CRLF_DOUBLE, HTTP_VERSION, PR_FILE_EXTENSION, PR_FOLDER}, servers::errors::ServerError, util::{connections::send_message, errors::UtilError, files::{create_directory, folder_exists}}};
use super::{features_pr::get_commits_pr, http_body::HttpBody, model::Model, status_code::StatusCode};

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
    match create_directory(pr_folder_path) 
    {
        Ok(_) => Ok(()),
        Err(_) => Err(ServerError::CreatePrFolderError),
    }
}



/// Envía una respuesta HTTP al cliente.
///
/// Esta función construye una respuesta HTTP con la versión y el código de estado proporcionados,
/// y la envía utilizando el escritor proporcionado.
///
/// # Argumentos
///
/// * `writer` - Un escritor que implementa el trait `Write` para enviar la respuesta.
/// * `status_code` - El código de estado HTTP que se debe incluir en la respuesta.
///
/// # Retornos
///
/// Retorna `Ok(())` si la respuesta se envía correctamente, o un `ServerError` si ocurre un error al enviar la respuesta.
///
/// # Errores
///
/// Retorna `ServerError::SendResponse` si falla al escribir la respuesta en el escritor.
pub fn send_response_http(writer: &mut dyn Write, status_code: &StatusCode, content_type: &str) -> Result<(), ServerError>{
    let response = format!("{} {}{}", HTTP_VERSION, status_code, CRLF);
    let error = UtilError::UtilFromServer("Error sending response".to_string());
    match send_message(writer, &response, error)
    {
        Ok(_) => {},
        Err(_) => return Err(ServerError::SendResponse(response)),
    };
    match status_code {
        StatusCode::Ok(Some(body)) => {
            // let body = HttpBody::convert_body_to_content_type(body.clone(), content_type)?;
            send_body_model(writer, body, content_type)
        },
        StatusCode::ValidationFailed(message)
        | StatusCode::InternalError(message)
        | StatusCode::ResourceNotFound(message)
        | StatusCode::Forbidden(message)
        | StatusCode::BadRequest(message) => {
            // let body = HttpBody::from_string(content_type, message, MESSAGE)?;
            let body = Model::Message(message.to_string());
            send_body_model(writer, &body, content_type)
        },
        _ => Ok(()) // Deberia enviar un CRLF
    }
}

fn send_body_model(writer: &mut dyn Write, model: &Model, content_type: &str) -> Result<(), ServerError> {
    // let (content_type, body_str) = body.get_content_type_and_body()?;
    let body_str = model.to_string(content_type);

    let message = match body_str.len()
    {
        0 => CRLF.to_string(),
        _ => format!("Content-Type: {}{}Content-Length: {}{}{}", content_type, CRLF,body_str.len(), CRLF_DOUBLE, body_str),
    };
    let error = UtilError::UtilFromServer("Error sending response body".to_string());
    match send_message(writer, &message, error)
    {
        Ok(_) => Ok(()),
        Err(_) => Err(ServerError::SendResponse(body_str)),
    }
}

/// Envía el cuerpo de una respuesta HTTP a través de un escritor.
///
/// Esta función toma un escritor y un cuerpo HTTP, obtiene el tipo de contenido y el cuerpo en forma de cadena,
/// y luego envía el cuerpo junto con los encabezados necesarios para una respuesta HTTP.
/// 
/// # Argumentos
///
/// * `writer` - Un escritor mutable que implementa el trait `Write`. Este escritor será utilizado para enviar la respuesta.
/// * `body` - Una referencia a un `HttpBody` que contiene el cuerpo de la respuesta a enviar.
///
/// # Errores
///
/// Esta función retornará un `ServerError` si ocurre un error al enviar el cuerpo de la respuesta.
/// Los errores pueden ser causados por problemas al obtener el tipo de contenido y el cuerpo,
/// o por fallos al escribir en el escritor proporcionado.
///
// fn send_body(writer: &mut dyn Write, body: &HttpBody) -> Result<(), ServerError> {
//     let (content_type, body_str) = body.get_content_type_and_body()?;
    
//     let message = match body_str.len()
//     {
//         0 => CRLF.to_string(),
//         _ => format!("Content-Type: {}{}Content-Length: {}{}{}", content_type, CRLF,body_str.len(), CRLF_DOUBLE, body_str),
//     };
//     let error = UtilError::UtilFromServer("Error sending response body".to_string());
//     match send_message(writer, &message, error)
//     {
//         Ok(_) => Ok(()),
//         Err(_) => Err(ServerError::SendResponse(body.to_string())),
//     }
// }

/// Valida si un repositorio existe en el directorio especificado.
///
/// # Argumentos
///
/// * `repo_name` - El nombre del repositorio a validar.
/// * `base_path` - La ruta base donde se encuentran los repositorios.
///
/// # Retorna
///
/// * `Ok(())` si el repositorio existe.
/// * `Err(ServerError::ResourceNotFound)` si el repositorio o su carpeta `.git` no existen.
///
/// # Errores
///
/// Esta función retornará `ServerError::ResourceNotFound` si el repositorio o su carpeta `.git` no existen.
pub fn valid_repository(repo_name: &str, base_path: &String) -> Result<(), ServerError> {
    let repo_directory = format!("{}/{}", base_path, repo_name);
    if !folder_exists(&repo_directory)
    {
        return Err(ServerError::ResourceNotFound("The repository does not exist.".to_string()));
    }
    let git = format!("{}/.git", repo_directory);
    if !folder_exists(&git)
    {
        return Err(ServerError::ResourceNotFound("The repository does not exist.".to_string()));
    }
    Ok(())
}

/// Obtiene el número del próximo pull request a partir de un archivo.
///
/// Si el archivo no existe, se crea y se inicializa en 1.
///
/// # Argumentos
///
/// * `file_path` - La ruta al archivo que almacena el número del próximo pull request.
///
/// # Errores
///
/// Retorna `ServerError::CreateNextPrFile` si hay un problema al crear el archivo.
/// Retorna `ServerError::ReadNextPrFile` si hay un problema al leer el archivo.
/// Retorna `ServerError::WriteNextPrFile` si hay un problema al escribir en el archivo.
/// 
pub fn get_next_pr_number(file_path: &str) -> Result<usize, ServerError> {
    // Abre el archivo para lectura y escritura, crea si no existe
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_path)
        .map_err(|_| ServerError::CreateNextPrFile)?;

    let mut content = String::new();

    // Lee el contenido del archivo
    file.read_to_string(&mut content).map_err(|_| ServerError::ReadNextPrFile)?;
    // Determina el número del próximo PR
    let next_pr_number = if content.trim().is_empty() {
        1 // Si el archivo está vacío, comienza con 1
    } else {
        parse_next_pr_number(&content)?
    };

    // Vacía el archivo y coloca el cursor al principio
    file.set_len(0).map_err(|_| ServerError::WriteNextPrFile)?;
    file.seek(SeekFrom::Start(0)).map_err(|_| ServerError::WriteNextPrFile)?;

    // Escribe el siguiente número en el archivo
    file.write_all((next_pr_number + 1).to_string().as_bytes())
        .map_err(|_| ServerError::WriteNextPrFile)?;

    Ok(next_pr_number)
}

fn parse_next_pr_number(content: &str) -> Result<usize, ServerError> {
    content.trim().parse::<usize>().map_err(|err: ParseIntError| {
        println!("Failed to parse number from content: {}. Error: {:?}", content, err);
        ServerError::ParseNumberPR("Failed to parse PR number".to_string())
    })
}

/// Valida si hay cambios entre las ramas `head` y `base`.
///
/// # Argumentos
///
/// * `repo_name` - El nombre del repositorio.
/// * `base_path` - La ruta base donde se encuentran los repositorios.
/// * `head` - La rama de origen.
/// * `base` - La rama de destino.
///
/// # Retornos
///
/// * `Ok(true)` - Si hay cambios entre `head` y `base`.
/// * `Ok(false)` - Si no hay cambios entre `head` y `base`.
/// * `Err(ServerError)` - Si ocurre un error durante la validación.
/// 
pub fn validate_branch_changes(repo_name: &str, base_path: &str, base: &str, head: &str) -> Result<bool, ServerError> {
    let directory = format!("{}/{}", base_path, repo_name);
    let result = get_commits_pr(&directory, base, head)?;

    Ok(!result.is_empty())
}

/// Guarda el cuerpo de un pull request en un archivo con un número de PR único.
///
/// Esta función guarda el cuerpo del pull request en un archivo con un nombre generado a partir del
/// número de pull request único, asegurando que el archivo se almacene en el formato adecuado.
///
/// # Argumentos
///
/// * `body` - El cuerpo de la solicitud HTTP que contiene los datos del pull request.
/// * `path` - La ruta del directorio donde se guardará el archivo del pull request.
/// * `pr_number` - El número de pull request que se usará para nombrar el archivo.
///
/// # Retornos
///
/// Devuelve `Ok(())` si el pull request se guarda correctamente.
/// Devuelve `Err(ServerError)` si ocurre un error durante el guardado del archivo.
/// 
pub fn save_pr_to_file(body: &HttpBody, path: &str, pr_number: usize) -> Result<(), ServerError> {
    let pr_file_path = format!("{}/{}{}", path, pr_number, PR_FILE_EXTENSION);
    body.save_body_to_file(&pr_file_path, APPLICATION_SERVER)?;
    Ok(())
}


/// Configura el directorio para los archivos de pull requests en un repositorio dado.
///
/// Esta función crea un directorio dentro del repositorio especificado para almacenar los archivos de
/// los pull requests. Si el directorio no existe, se crea. La función devuelve la ruta completa del
/// directorio de pull requests.
///
/// # Argumentos
///
/// * `repo_name` - El nombre del repositorio para el cual se configura el directorio de pull requests.
/// * `src` - La ruta base del directorio donde se encuentra el repositorio.
///
/// # Retornos
///
/// Devuelve `Ok(String)` con la ruta del directorio de pull requests si se crea o existe correctamente.
/// Devuelve `Err(StatusCode::InternalError)` si ocurre un error al crear el directorio.
/// 
pub fn setup_pr_directory(repo_name: &str, src: &String) -> Result<String, StatusCode> {
    let path = format!("{}/{}/{}", src, PR_FOLDER, repo_name);
    let directory = Path::new(&path);
    if create_directory(directory).is_err() {
        return Err(StatusCode::InternalError("Error creating the PR folder.".to_string()));
    }
    Ok(path)
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
