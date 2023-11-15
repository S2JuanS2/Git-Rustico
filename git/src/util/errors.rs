use std::fmt;

use crate::{commands::errors::CommandsError, errors::GitError};

#[derive(PartialEq, Eq)]
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
    InvalidRequestCommandMissingHost,
    InvalidPacketLineMissingLength,
    InvalidPacketLineLength,
    InvalidPacketLineReadData,
    InvalidPacketLineMissingNewline,
    HeaderPackFileReadSignature,
    HeaderPackFileReadVersion,
    HeaderPackFileReadNumberObjects,
    DataPackFiletReadObject,
    InvalidObjectType,
    ObjectDeserialization,
    EmptyDecompressionError,
    PackfileNegotiationReceiveNACK,
    InvalidPacketLineRequest,
    RequestInvalidHostFormat,
    InvalidRequestFlush,
    RepoNotFoundError(String),
    TypeInvalideference,
    ReferencesObtaining,
    HeadFolderNotFound,
    InvalidHeadReferenceFormat,
    HeadHashNotFound,
    FlushNotSentDiscoveryReferences,
    VersionNotSentDiscoveryReferences,
    UnexpectedRequestNotWant,
    InvalidRequestFormat(String),
    NegociacionExpectedDone,
    SendVersionPackfile,
    SendSignaturePackfile,
    GetObjectsPackfile,
    SendNACKPackfile,
    SendObjectPackfile,
    ObjectDeserializationPackfile,
    ChannelSendLog,
    UnexpectedRequestNotHave,
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
        UtilError::InvalidRequestCommandMissingCommand => write!(f, "InvalidRequestCommandCommandError: Solicitud sin comando"),
        UtilError::InvalidRequestCommandMissingPathname => write!(f, "InvalidRequestCommandPathnameError: No se encontro ruta de archivo para la solicitud."),
        UtilError::InvalidRequestCommandMissingHost => write!(f, "InvalidRequestCommandHostError: No se encontro nombre de host para la solicitud"),
        UtilError::InvalidPacketLineLength => write!(f, "InvalidPacketLineLengthError: Longitud de línea de paquete inválida."),
        UtilError::InvalidPacketLineMissingLength => write!(f, "InvalidPacketLineMissingLengthError: Falta la longitud de la línea de paquete."),
        UtilError::InvalidPacketLineReadData => write!(f, "InvalidPacketLineReadDataError: Error al leer los datos de la línea de paquete."),
        UtilError::InvalidPacketLineMissingNewline => write!(f, "InvalidPacketLineMissingNewlineError: Falta el carácter de nueva línea de la línea de paquete."),
        UtilError::HeaderPackFileReadSignature => write!(f, "HeaderPackFileReadSignatureError: Error al leer la firma del encabezado del paquete."),
        UtilError::HeaderPackFileReadVersion => write!(f, "HeaderPackFileReadVersionError: Error al leer la versión del encabezado del paquete."),
        UtilError::HeaderPackFileReadNumberObjects => write!(f, "HeaderPackFileReadNumberObjectsError: Error al leer el número de objetos del encabezado del paquete."),
        UtilError::DataPackFiletReadObject => write!(f, "DataPackFiletReadObjectError: Error al leer el objeto del paquete."),
        UtilError::InvalidObjectType => write!(f, "InvalidObjectTypeError: Tipo de objeto inválido."),
        UtilError::ObjectDeserialization => write!(f, "ObjectDeserializationError: Error al deserializar el objeto."),
        UtilError::EmptyDecompressionError => write!(f, "EmptyDecompressionError: Error al descomprimir el objeto, me dio un vector vacío."),
        UtilError::PackfileNegotiationReceiveNACK => write!(f, "PackfileNegotiationReceiveNACKError: Error al recibir el NACK."),
        UtilError::InvalidPacketLineRequest => write!(f, "InvalidPacketLineRequestError: Error al leer la solicitud de línea de paquete. No cumple con el formato establecido"),
        UtilError::RequestInvalidHostFormat => write!(f, "RequestInvalidHostFormatError: Error al leer la solicitud de línea de paquete. El formato del host es inválido"),
        UtilError::InvalidRequestFlush => write!(f, "InvalidRequestFlushError: Error al leer la solicitud de línea de paquete. La solicitud de flush es inválida"),
        UtilError::RepoNotFoundError(repo) => write!(f, "RepoNotFoundError: No se encontró el repositorio {}", repo),
        UtilError::TypeInvalideference => write!(f, "TypeInvalideferenceError: Tipo de referencia inválido."),
        UtilError::ReferencesObtaining => write!(f, "ReferencesObtainingError: Error al obtener las referencias del repositorio."),
        UtilError::HeadFolderNotFound => write!(f, "HeadFolderNotFoundError: No se encontró el directorio HEAD."),
        UtilError::InvalidHeadReferenceFormat => write!(f, "InvalidHeadReferenceFormatError: Formato de referencia HEAD inválido."),
        UtilError::HeadHashNotFound => write!(f, "HeadHashNotFoundError: No se encontró el hash del HEAD."),
        UtilError::FlushNotSentDiscoveryReferences => write!(f, "FlushNotSentDiscoveryReferencesError: No se envió el flush para la solicitud de descubrimiento de referencias."),
        UtilError::VersionNotSentDiscoveryReferences => write!(f, "VersionNotSentDiscoveryReferencesError: No se envió la versión para la solicitud de descubrimiento de referencias."),
        UtilError::UnexpectedRequestNotWant => write!(f, "UnexpectedRequestNotWantError: Se recibió una solicitud inesperada que no es want."),
        UtilError::InvalidRequestFormat(request) => write!(f, "InvalidRequestFormatError: Formato de solicitud inválido: {}", request),
        UtilError::NegociacionExpectedDone => write!(f, "NegociacionExpectedDoneError: Se esperaba un done en la negociación."),
        UtilError::SendVersionPackfile => write!(f, "SendVersionPackfileError: Error al enviar la versión del packfile."),
        UtilError::SendSignaturePackfile => write!(f, "SendSignaturePackfileError: Error al enviar la firma del packfile."),
        UtilError::GetObjectsPackfile => write!(f, "GetObjectsPackfileError: Error al obtener los objetos del packfile."),
        UtilError::SendNACKPackfile => write!(f, "SendNACKPackfileError: Error al enviar el NACK para preparar el packfile."),
        UtilError::SendObjectPackfile => write!(f, "SendObjectPackfileError: Error al enviar el objeto del packfile."),
        UtilError::ObjectDeserializationPackfile => write!(f, "ObjectDeserializationPackfileError: Error al deserializar el objeto para crear el packfile."),
        UtilError::ChannelSendLog => write!(f, "ChannelSendLogError: Error al enviar un mensaje de registro por el canal."),
        UtilError::UnexpectedRequestNotHave => write!(f, "UnexpectedRequestNotHaveError: Se recibió una solicitud inesperada que no es have."),
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
