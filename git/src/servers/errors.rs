use std::fmt;

use crate::{commands::errors::CommandsError, errors::GitError, util::errors::UtilError};

#[derive(Clone, PartialEq)]
pub enum ServerError {
    SeverFromUtil(String),
    SeverFromCommands(String),
    ServerConnection,
    ServerDebug,
    ReadHttpRequest,
    HttpParseBody,
    MethodNotAllowed,
    CreatePrFolderError,
    HttpNoOwnerFound,
    HttpNoRepoFound,
    SendResponse(String),
    InvalidGetPathError,
    InvalidPostPathError,
    InvalidPutPathError,
    InvalidPatchPathError,
    HttpVersionNotSupported,
    UnsupportedMediaType,
    MissingRequestLine,
    IncompleteRequestLine,
    HttpParseXmlBody,
    HttpParseYamlBody,
    HttpParseJsonBody,
    HttpFieldNotFound(String),
    ResourceNotFound(String),
    StoredFileParse,
}

fn format_error(error: &ServerError, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match error {
        ServerError::SeverFromUtil(e) => write!(f, "Error del servidor: {}", e),
        ServerError::SeverFromCommands(e) => write!(f, "Error del servidor: {}", e),
        ServerError::ServerConnection => write!(f, "Error de conexión del servidor, no se pudo iniciar el servidor."),
        ServerError::ServerDebug => write!(f, "Error de depuración del servidor. Si estas en entrega final sos un boludo por usar este error"),
        ServerError::ReadHttpRequest => write!(f, "Error al leer la solicitud HTTP del cliente."),
        ServerError::HttpParseBody => write!(f, "Error al parsear el cuerpo de la solicitud HTTP."),
        ServerError::MethodNotAllowed => write!(f, "Método HTTP no permitido."),
        ServerError::CreatePrFolderError => write!(f, "Error al crear la carpeta de PR."),
        ServerError::HttpNoOwnerFound => write!(f, "No se encontró el propietario del repositorio en la solicitud."),
        ServerError::HttpNoRepoFound => write!(f, "No se encontró el repositorio en la solicitud."),
        ServerError::SendResponse(e) => write!(f, "Error al enviar la respuesta HTTP: {}", e),
        ServerError::InvalidGetPathError => write!(f, "Ruta GET no válida."),
        ServerError::InvalidPostPathError => write!(f, "Ruta POST no válida."),
        ServerError::InvalidPutPathError => write!(f, "Ruta PUT no válida."),
        ServerError::InvalidPatchPathError => write!(f, "Ruta PATCH no válida."),
        ServerError::HttpVersionNotSupported => write!(f, "Versión HTTP no soportada. Solo se soporta HTTP/1.1."),
        ServerError::UnsupportedMediaType => write!(f, "Tipo de medio no soportado."),
        ServerError::MissingRequestLine => write!(f, "Línea de solicitud HTTP faltante."),
        ServerError::IncompleteRequestLine => write!(f, "Línea de solicitud HTTP incompleta."),
        ServerError::HttpParseXmlBody => write!(f, "Error al parsear el cuerpo XML de la solicitud HTTP."),
        ServerError::HttpParseYamlBody => write!(f, "Error al parsear el cuerpo YAML de la solicitud HTTP."),
        ServerError::HttpParseJsonBody => write!(f, "Error al parsear el cuerpo JSON de la solicitud HTTP."),
        ServerError::HttpFieldNotFound(e) => write!(f, "Campo no encontrado en el cuerpo de la solicitud HTTP: {}", e),
        ServerError::ResourceNotFound(e) => write!(f, "Recurso no encontrado: {}", e),
        ServerError::StoredFileParse => write!(f, "Error al parsear el archivo almacenado."),
    }
}

impl From<ServerError> for GitError {
    fn from(err: ServerError) -> Self {
        GitError::GitFromServerError(format!("{}", err))
    }
}

impl From<UtilError> for ServerError {
    fn from(error: UtilError) -> Self {
        ServerError::SeverFromUtil(format!("{}", error))
    }
}

impl From<CommandsError> for ServerError {
    fn from(error: CommandsError) -> Self {
        ServerError::SeverFromCommands(format!("{}", error))
    }
}


impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_error(self, f)
    }
}

impl fmt::Debug for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_error(self, f)
    }
}