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
}

fn format_error(error: &ServerError, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match error {
        ServerError::SeverFromUtil(e) => write!(f, "Error del servidor: {}", e),
        ServerError::SeverFromCommands(e) => write!(f, "Error del servidor: {}", e),
        ServerError::ServerConnection => write!(f, "Error de conexión del servidor, no se pudo iniciar el servidor."),
        ServerError::ServerDebug => write!(f, "Error de depuración del servidor. Si estas en entrega final sos un boludo por usar este error"),
        ServerError::ReadHttpRequest => write!(f, "Error al leer la solicitud HTTP del cliente."),
        ServerError::HttpParseBody => write!(f, "Error al parsear el cuerpo de la solicitud HTTP."),
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