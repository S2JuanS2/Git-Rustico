use std::fmt;

use crate::{errors::GitError, util::errors::UtilError};

pub enum CommandsError {
    CommandsFromUtil(String), // Para tener polimofismo con UtilError
    CloneMissingRepoInfo(String),
    CloneMissingRepo,
}

fn format_error(error: &CommandsError, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match error {
        CommandsError::CloneMissingRepo => write!(f, "CloneMissingRepo: Use: git clone <repositorio>"),
        CommandsError::CloneMissingRepoInfo(info) => write!(f, "{}\nMore info: {}", CommandsError::CloneMissingRepo, info),
        CommandsError::CommandsFromUtil(info) => write!(f, "{}", info),
        // AGregar más errores aquí
    }
}


impl From<CommandsError> for GitError {
    fn from(err: CommandsError) -> Self {
        GitError::GitFromCommandsError(format!("{}", err))
    }
}

impl From<UtilError> for CommandsError {
    fn from(error: UtilError) -> Self {
        CommandsError::CommandsFromUtil(format!("{}", error))
    }
}

// Esto no se toca
impl fmt::Display for CommandsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_error(self, f)
    }
}

// Esto no se toca
impl fmt::Debug for CommandsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_error(self, f)
    }
}