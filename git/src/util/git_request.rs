
use std::fmt;
use std::io::Read;

use crate::consts::END_OF_STRING;
use crate::util::pkt_line::add_length_prefix;

use super::errors::UtilError;
use super::pkt_line::{read_pkt_line, read_line_from_bytes};
use super::request_command::RequestCommand;


/// # `GitRequest`
///
/// Estructura que representa una solicitud de Git, que contiene un comando, una ruta y parámetros adicionales.
///
/// `GitRequest` encapsula los componentes de una solicitud Git, incluyendo el comando de la solicitud, la ruta del repositorio
/// y los parámetros adicionales.
///
/// ## Miembros
///
/// - `request_command`: Comando de la solicitud Git, representado por un tipo `RequestCommand`.
///
/// - `pathname`: Ruta del repositorio solicitado en la solicitud Git.
///
/// - `extra_parameters`: Parámetros adicionales proporcionados en la solicitud Git.
///
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
    pub fn read_git_request(reader: &mut dyn Read) -> Result<GitRequest, UtilError> {
        let data = read_pkt_line(reader)?;
        if data.is_empty() {
            return Err(UtilError::InvalidRequestFlush);
        }
        process_request_data(&data)
    }

    /// Crea una solicitud Git a partir de datos en bytes leídos, los bytes leidos
    /// deben tener formato pkt.
    /// Se utiliza para convertir datos de una solicitud en bytes a una estructura `GitRequest`.
    /// Está destinada a manejar los datos de solicitud ya formateados en bytes.
    ///
    /// # Argumentos
    ///
    /// * `bytes` - Datos de la solicitud Git formateados como bytes.
    ///
    /// # Retorno
    ///
    /// Devuelve un `Result` que contiene un `GitRequest` si la solicitud se procesa correctamente,
    /// o bien un error de tipo `UtilError` si la solicitud es inválida o no puede procesarse.
    ///
    pub fn create_from_bytes(bytes: &[u8]) -> Result<GitRequest, UtilError> {
        let data = read_line_from_bytes(bytes)?;
        if data.is_empty() {
            return Err(UtilError::InvalidRequestFlush);
        }
        process_request_data(data)
    }
    
    /// Crea una solicitud Git a partir de información detallada proporcionada.
    /// Se utiliza para construir una solicitud Git con información específica como el comando,
    /// la dirección del repositorio, la IP y el puerto del host.
    ///
    /// # Argumentos
    ///
    /// * `command` - Comando de solicitud de Git (RequestCommand).
    /// * `repo` - Nombre del repositorio.
    /// * `ip` - Dirección IP del host.
    /// * `port` - Puerto del host.
    ///
    /// # Retorno
    ///
    /// Devuelve una cadena de texto que representa la solicitud Git con la información proporcionada.
    ///
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
    /// use git::util::git_request::GitRequest;
    /// use git::util::request_command::RequestCommand;
    ///
    /// let command = RequestCommand::UploadPack;
    /// let repo = "mi-repositorio".to_string();
    /// let ip = "127.0.0.1".to_string();
    /// let port = "22".to_string();
    ///
    /// let git_request = GitRequest::generate_request_string(command, repo, ip, port);
    ///
    /// // Verificar el resultado esperado
    /// assert_eq!(git_request, "0036git-upload-pack /mi-repositorio\0host=127.0.0.1:22\0");
    /// ```
    /// 
    /// ## Retorno
    ///
    /// Una line pkt que representa la solicitud Git formateada.
    pub fn generate_request_string(
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
    
    pub fn execute(&self) -> Result<(), UtilError> {
        match self.request_command {
            RequestCommand::UploadPack => {
                println!("UploadPack");
                Ok(())
            }
            RequestCommand::ReceivePack => {
                println!("ReceivePack");
                println!("Funcion aun no implementada");
                Ok(())
            }
            RequestCommand::UploadArchive => {
                println!("Funcion aun no implementada");
                println!("UploadArchive");
                Ok(())
            }
        }
    }
}

/// Procesa los datos de una solicitud Git y los convierte en una estructura `GitRequest`.
/// Esta función toma los datos de la solicitud Git y los divide en comandos y argumentos.
///
/// # Argumentos
///
/// * `data` - Los datos de la solicitud Git que se van a procesar.
///
/// # Retorno
///
/// Devuelve un `Result` que contiene un `GitRequest` si la solicitud se procesa correctamente,
/// o bien un error de tipo `UtilError` si la solicitud es inválida o no puede procesarse.
///
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



/// Obtiene los componentes de una solicitud Git y los retorna como tupla.
/// Toma los bytes de una solicitud Git y los separa en sus diferentes componentes,
/// devolviendo una tupla que contiene el pathname y los parámetros adicionales.
///
/// # Argumentos
///
/// * `bytes` - Bytes de la solicitud Git que se van a dividir en componentes.
///
/// # Retorno
///
/// Devuelve un `Result` que contiene una tupla con los componentes si la solicitud se procesa correctamente,
/// o bien un error de tipo `UtilError` si la solicitud es inválida o no puede procesarse.
///
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
    fn test_generate_request_string_upload_pack() {
        let message = GitRequest::generate_request_string(
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
    fn test_generate_request_string_receive_pack() {
        let message = GitRequest::generate_request_string(
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
    fn test_generate_request_string_upload_archive() {
        let message = GitRequest::generate_request_string(
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
