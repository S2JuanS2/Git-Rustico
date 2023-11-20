use std::fmt;

use crate::{errors::GitError, util::errors::UtilError};

pub enum CommandsError {
    CommandsFromUtil(String), // Para tener polimofismo con UtilError
    CloneMissingRepo,
    CommitEmptyIndex,
    InvalidArgumentCountFetchError,
    CloneMissingRepoError,
    RepositoryNotInitialized,
    CreateGitConfig,
    FileNotFoundConfig,
    MissingUrlConfig,
    InvalidArgumentCountPull,
    RemotoNotInitialized,
}

fn format_error(error: &CommandsError, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match error {
        CommandsError::CommandsFromUtil(info) => write!(f, "{}", info),
        CommandsError::CloneMissingRepo => {
            write!(f, "CloneMissingRepo: Use: git clone <repositorio>")
        }
        CommandsError::CommitEmptyIndex => write!(f, "Nada al que hacer Commit"),
        CommandsError::InvalidArgumentCountFetchError => {
            write!(f, "InvalidArgumentCountFetchError: Use: git fetch")
        }
        CommandsError::CloneMissingRepoError => {
            write!(f, "CloneMissingRepoError: Use: git clone <repositorio>")
        }
        CommandsError::RepositoryNotInitialized => write!(f, "RepositoryNotInitialized: Use: git init"),
        CommandsError::CreateGitConfig => write!(f, "CreateGitConfig: No se pudo crear el archivo de configuración de Git"),
        CommandsError::FileNotFoundConfig => write!(f, "FileNotFoundConfig: No se encontró el archivo de configuración de Git"),
        CommandsError::MissingUrlConfig => write!(f, "MissingUrlConfig: No se encontró la URL del repositorio remoto en el archivo de configuración de Git"),
        CommandsError::InvalidArgumentCountPull => write!(f, "InvalidArgumentCountPull: Use: git pull"),
        CommandsError::RemotoNotInitialized => write!(f, "RemotoNotInitialized: No se ha inicializado el repositorio remoto"),
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

impl fmt::Display for CommandsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_error(self, f)
    }
}

impl fmt::Debug for CommandsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_error(self, f)
    }
}
