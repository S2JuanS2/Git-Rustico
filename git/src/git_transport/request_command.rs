use std::fmt;

use crate::util::errors::UtilError;


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
    pub fn to_string(&self) -> &str {
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


#[cfg(test)]
mod request_command_tests {
    use super::*;

    #[test]
    fn display_upload_pack() {
        assert_eq!(format!("{}", RequestCommand::UploadPack), "Upload Pack");
    }

    #[test]
    fn display_receive_pack() {
        assert_eq!(format!("{}", RequestCommand::ReceivePack), "Receive Pack");
    }

    #[test]
    fn display_upload_archive() {
        assert_eq!(format!("{}", RequestCommand::UploadArchive), "Upload Archive");
    }

    #[test]
    fn to_string_upload_pack() {
        assert_eq!(RequestCommand::UploadPack.to_string(), "git-upload-pack");
    }

    #[test]
    fn to_string_receive_pack() {
        assert_eq!(RequestCommand::ReceivePack.to_string(), "git-receive-pack");
    }

    #[test]
    fn to_string_upload_archive() {
        assert_eq!(RequestCommand::UploadArchive.to_string(), "git-upload-archive");
    }

    #[test]
    fn from_string_upload_pack() {
        let data = b"git-upload-pack";
        assert_eq!(
            RequestCommand::from_string(data),
            Ok(RequestCommand::UploadPack)
        );
    }

    #[test]
    fn from_string_receive_pack() {
        let data = b"git-receive-pack";
        assert_eq!(
            RequestCommand::from_string(data),
            Ok(RequestCommand::ReceivePack)
        );
    }

    #[test]
    fn from_string_upload_archive() {
        let data = b"git-upload-archive";
        assert_eq!(
            RequestCommand::from_string(data),
            Ok(RequestCommand::UploadArchive)
        );
    }

    #[test]
    fn from_string_invalid() {
        let data = b"invalid-command";
        assert_eq!(
            RequestCommand::from_string(data),
            Err(UtilError::InvalidRequestCommand)
        );
    }
}
