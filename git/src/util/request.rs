use crate::consts::END_OF_STRING;
use crate::util::pkt_line::add_length_prefix;

use super::errors::UtilError;

/// Enumeración `RequestCommand` representa los comandos de solicitud en un protocolo Git.
///
/// Esta enumeración define tres comandos utilizados en las operaciones de Git.
///
/// - `UploadPack`: Comando para solicitar una transferencia de paquetes a través de "git-upload-pack".
/// - `ReceivePack`: Comando para solicitar una recepción de paquetes a través de "git-receive-pack".
/// - `UploadArchive`: Comando para solicitar la carga de un archivo de almacenamiento a través de "git-upload-archive".
///
pub enum RequestCommand {
    UploadPack,
    ReceivePack,
    UploadArchive,
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



pub struct GitRequest {
    pub request_command: RequestCommand,
    pub pathname: String,
    pub host_parameter: Option<String>,
    pub extra_parameters: Vec<String>,
}

impl GitRequest {
    pub fn read_git_proto_request(data: &[u8]) -> Result<GitRequest, UtilError> {

        let mut parts = data.split(|&byte| byte == 0);
        
        let request_command = match parts.next()
        {
            Some(command) => command,
            None => return Err(UtilError::InvalidRequestCommandInfo("No se pudo leer el comando de solicitud.".to_string())),
        };
        let pathname = match parts.next()
        {
            Some(path) => path,
            None => return Err(UtilError::InvalidRequestCommandInfo("No se pudo leer el nombre del repositorio.".to_string())),
        };
    
        let host_parameter = parts.next();
        let extra_parameters = parts.collect::<Vec<_>>();
    
        Ok(GitRequest {
            request_command: RequestCommand::from_string(request_command)?,
            pathname: String::from_utf8_lossy(pathname).trim().to_string(),
            host_parameter: host_parameter.map(|p| String::from_utf8_lossy(p).trim().to_string()),
            extra_parameters: extra_parameters
                .iter()
                .map(|p| String::from_utf8_lossy(p).trim().to_string())
                .collect(),
        })
    }

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
}
