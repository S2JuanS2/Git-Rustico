use std::fmt::{self};

use crate::{commands::errors::CommandsError, errors::GitError, servers::errors::ServerError};

#[derive(PartialEq, Eq, Clone)]
pub enum UtilError {
    UtilFromCommands(String), // Para tener polimorfismo con CommandsError
    UtilFromServer(String),   // Para tener polimorfismo con ServerError
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
    PackfileNegotiationReceiveNAK,
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
    SendNAKPackfile,
    SendObjectPackfile,
    ObjectDeserializationPackfile,
    ChannelSendLog,
    UnexpectedRequestNotHave,
    SendNAKConfirmReferences,
    ReceiveDoneConfRefs,
    SendLastACKConf,
    InvalidACKFormat(String),
    UnexpectedACKNotACK,
    ExpectedAckMissing,
    ExpectedHashInAckResponse,
    ExpectedStatusInAckResponse,
    InvalidHashInAckResponse,
    ExpectedStatusContinueInAckResponse,
    CreateDir(String),
    OpenFileError,
    ReadFileError,
    CreateFileError,
    DeleteFileError,
    ReadDirError,
    RemoveFileError,
    WriteFileError,
    CopyFileError,
    CreateDirError,
    VisitDirectoryError,
    DirEntryError,
    InvalidObjectLength,
    GetLocalReferences,
    MultiAckNotSupported,
    ServerCapabilitiesNotSupported,
    SendFlushCancelConnection,
    CurrentBranchNotFound,
    BranchNotFound(String),
    SendMessageReferenceUpdate,
    ObjectSerialization,
    SendSha1Packfile,
    ReceiveReferenceUpdateRequest,
    InvalidReferenceUpdateRequest,
    InvalidReferencePath,
    ConnectionIsTerminated,
    SendStatusUpdateRequest,
    CloseConnection,
    NotDirectory,
}

fn format_error(error: &UtilError, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match error {
        UtilError::UtilFromCommands(info) => write!(f, "{}", info),
        UtilError::UtilFromServer(info) => write!(f, "{}", info),
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
        UtilError::PackfileNegotiationReceiveNAK => write!(f, "PackfileNegotiationReceiveNAKError: Error al recibir el NAK."),
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
        UtilError::SendNAKPackfile => write!(f, "SendNAKPackfileError: Error al enviar el NAK para preparar el packfile."),
        UtilError::SendObjectPackfile => write!(f, "SendObjectPackfileError: Error al enviar el objeto del packfile."),
        UtilError::ObjectDeserializationPackfile => write!(f, "ObjectDeserializationPackfileError: Error al deserializar el objeto para crear el packfile."),
        UtilError::ChannelSendLog => write!(f, "ChannelSendLogError: Error al enviar un mensaje de registro por el canal."),
        UtilError::UnexpectedRequestNotHave => write!(f, "UnexpectedRequestNotHaveError: Se recibió una solicitud inesperada que no es have."),
        UtilError::SendNAKConfirmReferences => write!(f, "SendNAKConfirmReferencesError: Error al enviar el NAK para confirmar las referencias."),
        UtilError::ReceiveDoneConfRefs => write!(f, "ReceiveDoneConfRefsError: Error al recibir el done para confirmar las referencias."),
        UtilError::SendLastACKConf => write!(f, "SendLastACKConfError: Error al enviar el último ACK para confirmar las referencias."),
        UtilError::InvalidACKFormat(info) => write!(f, "InvalidACKFormatError: Formato de ACK inválido. \n Se recibió: {} \n Se esperaba: ACK <hash> <status>", info),
        UtilError::UnexpectedACKNotACK => write!(f, "UnexpectedACKNotACKError: Se recibió un ACK inesperado."),
        UtilError::ExpectedAckMissing => write!(f, "ExpectedAckMissingError: Se esperaba un ACK."),
        UtilError::ExpectedHashInAckResponse => write!(f, "ExpectedHashInAckResponseError: Se esperaba un hash en la respuesta del ACK."),
        UtilError::ExpectedStatusInAckResponse => write!(f, "ExpectedStatusInAckResponseError: Se esperaba un status en la respuesta del ACK."),
        UtilError::InvalidHashInAckResponse => write!(f, "InvalidHashInAckResponseError: Hash inválido en la respuesta del ACK."),
        UtilError::ExpectedStatusContinueInAckResponse => write!(f, "ExpectedStatusContinueInAckResponseError: Se esperaba un status continue en la respuesta del ACK."),
        UtilError::CreateDir(info) => writeln!(f, "CreateDirError: Error al crear el directorio {}. ", info),
        UtilError::OpenFileError => write!(f, "UtilError: No se pudo abrir el archivo"),
        UtilError::ReadFileError => write!(f, "No se pudo leer el archivo"),
        UtilError::CreateFileError => write!(f, "No se pudo crear el archivo"),
        UtilError::RemoveFileError => write!(f, "No se pudo eliminar el archivo"),
        UtilError::WriteFileError => write!(f, "Fallo al escribir en el archivo"),
        UtilError::CopyFileError => write!(f, "Fallo al copiar el archivo"),
        UtilError::CreateDirError => write!(f, "Fallo al crear el directorio"),
        UtilError::ReadDirError => write!(f, "Falló al leer el directorio"),
        UtilError::DirEntryError => write!(f, "Falló al obtener la entrada del directorio"),
        UtilError::VisitDirectoryError => write!(f, "No se pudo recorrer el directorio"),
        UtilError::InvalidObjectLength => write!(f, "Fallo al leer en el header, se leyo una longitud de objecto invalido"),
        UtilError::DeleteFileError => write!(f, "No se pudo encontrar el archivo"),
        UtilError::GetLocalReferences => write!(f, "GetLocalReferences: No se pudieron obtener las referencias locales"),
        UtilError::MultiAckNotSupported => write!(f, "MultiAckNotSupported: El servidor no soporta multi_ack"),
        UtilError::ServerCapabilitiesNotSupported => write!(f, "ServerCapabilitiesNotSupported: El servidor no soporta mis capacidades"),
        UtilError::SendFlushCancelConnection => write!(f, "SendFlushCancelConnection: Error al enviar el flush para terminar la conexión."),
        UtilError::CurrentBranchNotFound => write!(f, "CurrentBranchNotFound: No se encontró la rama actual."),
        UtilError::BranchNotFound(s) => write!(f, "BranchNotFound: No se encontró la rama: {}", s),
        UtilError::SendMessageReferenceUpdate => write!(f, "SendMessageReferenceUpdate: Error al enviar el mensaje de actualización de referencia."),
        UtilError::ObjectSerialization => write!(f, "ObjectSerialization: Error al serializar el objeto."),
        UtilError::SendSha1Packfile => write!(f, "SendSha1Packfile: Error al enviar el sha1 del packfile."),
        UtilError::ReceiveReferenceUpdateRequest => write!(f, "ReceiveReferenceUpdateRequest: Error al recibir la solicitud de actualización de referencia."),
        UtilError::InvalidReferenceUpdateRequest => write!(f, "InvalidReferenceUpdateRequest: Solicitud de actualización de referencia inválida."),
        UtilError::InvalidReferencePath => write!(f, "InvalidReferencePath: Ruta de referencia inválida."),
        UtilError::ConnectionIsTerminated => write!(f, "ConnectionIsTerminated: La conexión fue terminada."),
        UtilError::SendStatusUpdateRequest => write!(f, "SendStatusUpdateRequest: Error al enviar la solicitud de actualización de estado."),
        UtilError::CloseConnection => write!(f, "CloseConnection: Error al cerrar la conexión."),
        UtilError::NotDirectory => write!(f, "NotDirectory: No es un directorio."),

    }
}

impl From<CommandsError> for UtilError {
    fn from(error: CommandsError) -> Self {
        UtilError::UtilFromCommands(format!("{}", error))
    }
}

impl From<ServerError> for UtilError {
    fn from(error: ServerError) -> Self {
        UtilError::UtilFromServer(format!("{}", error))
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
