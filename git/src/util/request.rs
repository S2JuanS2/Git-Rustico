
use std::fmt;
use std::io::Read;

use crate::consts::END_OF_STRING;
use crate::util::pkt_line::add_length_prefix;

use super::errors::UtilError;
use super::pkt_line::{read_pkt_line, read_line_from_bytes};

/// Enumeración `RequestCommand` representa los comandos de solicitud en un protocolo Git.
///
/// Esta enumeración define tres comandos utilizados en las operaciones de Git.
///
/// - `UploadPack`: Comando para solicitar una transferencia de paquetes a través de "git-upload-pack".
/// - `ReceivePack`: Comando para solicitar una recepción de paquetes a través de "git-receive-pack".
/// - `UploadArchive`: Comando para solicitar la carga de un archivo de almacenamiento a través de "git-upload-archive".
///
#[derive(Debug, PartialEq, Eq)]
pub enum RequestCommand {
    UploadPack,
    ReceivePack,
    UploadArchive,
}

impl fmt::Display for RequestCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let command = match self {
            RequestCommand::UploadPack => "Upload Pack",
            RequestCommand::ReceivePack => "Receive Pack",
            RequestCommand::UploadArchive => "Upload Archive",
        };
        write!(f, "{}", command)
    }
}

impl RequestCommand {
    /// Convierte un valor de `RequestCommand` en su representación de cadena.
    fn to_string(&self) -> &str {
        match self {
            RequestCommand::UploadPack => "git-upload-pack",
            RequestCommand::ReceivePack => "git-receive-pack",
            RequestCommand::UploadArchive => "git-upload-archive",
        }
    }

    pub fn from_string(data: &[u8]) -> Result<RequestCommand, UtilError> {
        let binding = String::from_utf8_lossy(data);
        let command = binding.trim();
        match command {
            "git-upload-pack" => Ok(RequestCommand::UploadPack),
            "git-receive-pack" => Ok(RequestCommand::ReceivePack),
            "git-upload-archive" => Ok(RequestCommand::UploadArchive),
            _ => Err(UtilError::InvalidRequestCommand),
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct GitRequest {
    pub request_command: RequestCommand,
    pub pathname: String,
    pub extra_parameters: Vec<String>,
}
impl fmt::Display for GitRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Request Command: {}\nPathname: {}\nExtra Parameters: {:?}",
            self.request_command, self.pathname, self.extra_parameters
        )
    }
}

impl GitRequest {
    /// Lee y procesa una solicitud de Git a partir de datos leídos de un flujo de lectura.
    /// Se utiliza para leer y procesar la solicitud de un cliente Git desde un flujo de lectura.
    /// Se espera que la solicitud se reciba como un paquete de líneas de Git y, por lo tanto,
    /// se utiliza la función `read_pkt_line` para extraer los datos.
    ///
    /// # Argumentos
    ///
    /// * `listener` - Un flujo de lectura que se utiliza para obtener datos de una solicitud Git.
    ///
    /// # Retorno
    ///
    /// Devuelve un `Result` que contiene un `GitRequest` si la solicitud se procesa correctamente,
    /// o bien un error de tipo `UtilError` si la solicitud es inválida o no puede procesarse.
    ///
    pub fn read_git_request(listener: &mut dyn Read) -> Result<GitRequest, UtilError> {
        let data = read_pkt_line(listener)?;

        process_request_data(&data)
    }

    pub fn create_from_bytes(bytes: &[u8]) -> Result<GitRequest, UtilError> {
        let data = read_line_from_bytes(bytes)?;

        process_request_data(data)
    }
    
    pub fn create_from_command(
        command: RequestCommand,
        repo: String,
        ip: String,
        port: String,
    ) -> String {
        let mut len: usize = 0;

        let command = format!("{} ", command.to_string());
        len += command.len();

        let project = format!("/{}{}", repo, END_OF_STRING);
        len += project.len(); // El len cuenta el END_OF_STRING

        let host = format!("host={}:{}{}", ip, port, END_OF_STRING);
        len += host.len(); // El len cuenta el END_OF_STRING

        let message = format!("{}{}{}", command, project, host);
        add_length_prefix(&message, len)
    }
}

fn process_request_data(data: &[u8]) -> Result<GitRequest, UtilError> {
    let (first_part, second_part) = if let Some(idx) = data.iter().position(|&byte| byte == b' ') {
        let (first, second) = data.split_at(idx);
        (first, &second[1..])
    } else {
        return Err(UtilError::InvalidRequestCommand);
    };

    let request_command = RequestCommand::from_string(first_part)?;

    get_components_request(second_part)
        .map(|(pathname, extra_parameters)| GitRequest {
            request_command,
            pathname: String::from_utf8_lossy(pathname).trim().to_string(),
            extra_parameters,
        })
}

/// Crea una solicitud Git con los datos especificados.
///
/// Esta función genera una solicitud Git formateada como una cadena, basada en el comando de solicitud,
/// el repositorio, la dirección IP y el puerto proporcionados como argumentos. La solicitud incluye
/// información sobre el comando, el proyecto (repositorio) y el host (IP y puerto).
///
/// ## Argumentos
///
/// - `command`: El comando de solicitud Git (`RequestCommand`) que se utilizará.
/// - `repo`: El nombre del repositorio en el que se realizará la solicitud.
/// - `ip`: La dirección IP del host al que se enviará la solicitud.
/// - `port`: El puerto en el que se realizará la conexión con el host.
///
/// ## Ejemplo
///
/// ```
/// use git::util::request::{create_git_request, RequestCommand};
///
/// let command = RequestCommand::UploadPack;
/// let repo = "mi-repositorio".to_string();
/// let ip = "127.0.0.1".to_string();
/// let port = "22".to_string();
///
/// let git_request = create_git_request(command, repo, ip, port);
///
/// // Verificar el resultado esperado
/// assert_eq!(git_request, "0036git-upload-pack /mi-repositorio\0host=127.0.0.1:22\0");
/// ```
///
/// ## Retorno
///
/// Una line pkt que representa la solicitud Git formateada.
pub fn create_git_request(
    command: RequestCommand,
    repo: String,
    ip: String,
    port: String,
) -> String {
    let mut len: usize = 0;

    let command = format!("{} ", command.to_string());
    len += command.len();

    let project = format!("/{}{}", repo, END_OF_STRING);
    len += project.len(); // El len cuenta el END_OF_STRING

    let host = format!("host={}:{}{}", ip, port, END_OF_STRING);
    len += host.len(); // El len cuenta el END_OF_STRING

    let message = format!("{}{}{}", command, project, host);
    add_length_prefix(&message, len)
}

// fn get_data_pkt(bytes: &[u8]) -> Result<&[u8], UtilError> {
//     match read_line_from_bytes(&bytes) {
//         Ok(data) => Ok(data),
//         Err(_) => Err(UtilError::InvalidPacketLineRequest),
//     }
// }

fn get_components_request(bytes: &[u8]) -> Result<(&[u8], Vec<String>), UtilError> {
    let mut components = bytes.split(|&byte| byte == 0);

    let pathname = match components.next() {
        Some(path) => path,
        None => return Err(UtilError::InvalidRequestCommandMissingPathname),
    };

    let extra_parameters = components.collect::<Vec<_>>();
    Ok((
        pathname,
        extra_parameters
            .iter()
            .map(|p| String::from_utf8_lossy(p).trim().to_string())
            .collect(),
    ))
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_git_request_upload_pack() {
        let message = create_git_request(
            RequestCommand::UploadPack,
            "project.git".to_string(),
            "myserver.com".to_string(),
            "9418".to_string(),
        );
        assert_eq!(
            message,
            "0038git-upload-pack /project.git\0host=myserver.com:9418\0"
        );
    }

    #[test]
    fn test_create_git_request_receive_pack() {
        let message = create_git_request(
            RequestCommand::ReceivePack,
            "project.git".to_string(),
            "127.0.0.2".to_string(),
            "12030".to_string(),
        );
        assert_eq!(
            message,
            "0037git-receive-pack /project.git\0host=127.0.0.2:12030\0"
        );
    }

    #[test]
    fn test_create_git_request_upload_archive() {
        let message = create_git_request(
            RequestCommand::UploadArchive,
            "project.git".to_string(),
            "250.250.250.250".to_string(),
            "8080".to_string(),
        );
        assert_eq!(
            message,
            "003egit-upload-archive /project.git\0host=250.250.250.250:8080\0"
        );
    }

        #[test]
        fn test_git_request_new_with_valid_format() {
            // Datos de entrada válidos con un espacio
            let input = b"003agit-upload-pack /schacon/gitbook.git\0host=example.com\0";
            let result = GitRequest::create_from_bytes(input);
            assert!(result.is_ok()); // Comprobar que el resultado es Ok
        }

        #[test]
        fn test_git_request_new_with_invalid_format() {
            // Datos de entrada sin espacio entre command y pathname
            let input_no_space = b"003agit-upload-pack/schacon/gitbook.git\0host=example.com\0";
            let result_no_space = GitRequest::create_from_bytes(input_no_space);
            assert!(result_no_space.is_err()); // Comprobar que el resultado es un error
        }

        #[test]
        fn test_git_request_new_with_invalid_lenght() {
            let input_no_space = b"013agit-upload-pack/schacon/gitbook.git\0host=example.com\0";
            let result_no_space = GitRequest::create_from_bytes(input_no_space);
            assert!(result_no_space.is_err()); // Comprobar que el resultado es un error
        }
        #[test]
        fn test_git_request_new_with_valid_data() -> Result<(), UtilError>{
            // Datos de entrada con un comando no válido
            let input_invalid_command = b"003agit-upload-pack /schacon/gitbook.git\0host=example.com\0";
            let request = GitRequest::create_from_bytes(input_invalid_command)?;
            assert!(request.request_command == RequestCommand::UploadPack);
            assert!(request.pathname == "/schacon/gitbook.git");
            assert!(vec![String::from("host=example.com")].eq(&request.extra_parameters));
            Ok(())
        }
}
