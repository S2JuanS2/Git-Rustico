use std::fmt;

use crate::{errors::GitError, commands::errors::CommandsError};

pub enum UtilError {
    UtilFromCommands(String), // Para tener polimorfismo con CommandsError
    InvalidPacketLine,
    ServerConnection,
    ClientConnection,
    LogOutputSync,
    LogMessageSend,
    LogOutputOpen,
    InvalidRequestCommand,
    UploadRequest,
    GenericError, // Para los tests
    ReferenceDiscovey,
    InvalidVersionNumber,
    InvalidObjectId,
    InvalidServerReference,
    UploadRequestFlush,
    UploadRequestDone,
    InvalidRequestCommandMissingCommand,
    InvalidRequestCommandMissingPathname,
    InvalidPacketLineMissingLength,
    InvalidPacketLineLength,
    InvalidPacketLineReadData,
    InvalidPacketLineMissingNewline,
}

fn format_error(error: &UtilError, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match error {
        UtilError::UtilFromCommands(info) => write!(f, "{}", info),
        UtilError::InvalidPacketLine => write!(f, "InvalidPacketLineError: Error al leer una línea de paquete."),
        UtilError::ServerConnection => write!(f, "ServerConnectionError: Error al iniciar el servidor."),
        UtilError::ClientConnection => write!(f, "ClientConnectionError: Error al iniciar el cliente."),
        UtilError::LogOutputSync => write!(f, "LogOutputSyncError: Error al sincronizar la salida de registro."),
        UtilError::LogMessageSend => write!(f, "LogMessageSendError: Error al enviar un mensaje de registro."),
        UtilError::LogOutputOpen => write!(f, "LogOutputOpenError: Error al abrir la salida de registro."),
        UtilError::InvalidRequestCommand => write!(f, "InvalidRequestCommandError: Comando de solicitud inválido."),
        UtilError::UploadRequest => write!(f, "UploadRequestError: Error al enviar una solicitud de carga."),
        UtilError::GenericError => write!(f, "GenericError: Error genérico para los tests."),
        UtilError::ReferenceDiscovey => write!(f, "ReferenceDiscoveyError: Error al realizar el descubrimiento de referencias."),
        UtilError::InvalidVersionNumber => write!(f, "InvalidVersionNumberError: Error al leer el número de versión del paquete."),
        UtilError::InvalidObjectId => write!(f, "InvalidObjectIdError: Error al leer el identificador de objeto."),
        UtilError::InvalidServerReference => write!(f, "InvalidServerReferenceError: Error al leer la referencia del servidor."),
        UtilError::UploadRequestFlush => write!(f, "UploadRequestFlushError: Error al enviar el flush."),
        UtilError::UploadRequestDone => write!(f, "UploadRequestDoneError: Error al enviar el done."),
        UtilError::InvalidRequestCommandMissingCommand => write!(f, "InvalidRequestCommandCommandError: Comando de solicitud inválido."),
        UtilError::InvalidRequestCommandMissingPathname => write!(f, "InvalidRequestCommandPathnameError: Nombre de ruta de archivo inválido."),
        UtilError::InvalidPacketLineLength => write!(f, "InvalidPacketLineLengthError: Longitud de línea de paquete inválida."),
        UtilError::InvalidPacketLineMissingLength => write!(f, "InvalidPacketLineMissingLengthError: Falta la longitud de la línea de paquete."),
        UtilError::InvalidPacketLineReadData => write!(f, "InvalidPacketLineReadDataError: Error al leer los datos de la línea de paquete."),
        UtilError::InvalidPacketLineMissingNewline => write!(f, "InvalidPacketLineMissingNewlineError: Falta el carácter de nueva línea de la línea de paquete."),
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