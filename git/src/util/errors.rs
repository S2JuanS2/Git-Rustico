use std::fmt;

use crate::{errors::GitError, commands::errors::CommandsError};

pub enum UtilError {
    UtilFromCommands(String), // Para tener polimorfismo con CommandsError
    InvalidPacketLineInfo(String),
    InvalidPacketLine,
    ServerConnection,
    ClientConnection,
    LogOutputSync,
    LogMessageSend,
    LogOutputOpen,
    InvalidRequestCommand,
    InvalidRequestCommandInfo(String),
    UploadRequest,
    UploadRequestInfo(String),
    GenericError, // Para los tests
    ReferenceDiscovey,
}

fn format_error(error: &UtilError, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match error {
        UtilError::UtilFromCommands(info) => write!(f, "{}", info),
        UtilError::InvalidPacketLine => write!(f, "InvalidPacketLineError: Error al leer una línea de paquete."),
        UtilError::InvalidPacketLineInfo(info) => write!(f, "{}\nMore info: {}", UtilError::InvalidPacketLine, info),
        UtilError::ServerConnection => write!(f, "ServerConnectionError: Error al iniciar el servidor."),
        UtilError::ClientConnection => write!(f, "ClientConnectionError: Error al iniciar el cliente."),
        UtilError::LogOutputSync => write!(f, "LogOutputSyncError: Error al sincronizar la salida de registro."),
        UtilError::LogMessageSend => write!(f, "LogMessageSendError: Error al enviar un mensaje de registro."),
        UtilError::LogOutputOpen => write!(f, "LogOutputOpenError: Error al abrir la salida de registro."),
        UtilError::InvalidRequestCommand => write!(f, "InvalidRequestCommandError: Comando de solicitud inválido."),
        UtilError::InvalidRequestCommandInfo(info) => write!(f, "{}\nMore info: {}", UtilError::InvalidRequestCommand, info),
        UtilError::UploadRequest => write!(f, "UploadRequestError: Error al enviar una solicitud de carga."),
        UtilError::UploadRequestInfo(info) => write!(f, "{}\nMore info: {}", UtilError::UploadRequest, info),
        UtilError::GenericError => write!(f, "GenericError: Error genérico para los tests."),
        UtilError::ReferenceDiscovey => write!(f, "ReferenceDiscoveyError: Error al realizar el descubrimiento de referencias."),
        // AGregar más errores aquí
    }
}


impl From<CommandsError> for UtilError {
    fn from(error: CommandsError) -> Self {
        UtilError::UtilFromCommands(format!("{}", error))
    }
}

impl From<UtilError> for GitError {
    fn from(err: UtilError) -> Self {
        GitError::GitFromUtilError(format!("{}", err))
    }
}

// Esto no se toca
impl fmt::Display for UtilError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_error(self, f)
    }
}

// Esto no se toca
impl fmt::Debug for UtilError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_error(self, f)
    }
}