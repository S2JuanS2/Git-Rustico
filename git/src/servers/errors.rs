use std::fmt;

use crate::{commands::errors::CommandsError, errors::GitError, util::errors::UtilError};

#[derive(Clone, PartialEq)]
pub enum ServerError {
    SeverFromUtil(String),
    SeverFromCommands(String),
}

fn format_error(error: &ServerError, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match error {
        ServerError::SeverFromUtil(e) => write!(f, "Error del servidor: {}", e),
        ServerError::SeverFromCommands(e) => write!(f, "Error del servidor: {}", e),
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