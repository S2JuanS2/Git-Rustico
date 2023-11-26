use std::fmt;

use crate::{errors::GitError, util::errors::UtilError};

#[derive(Clone, PartialEq)]
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
    CreateFetchHEAD,
    ReadFetchHEAD,
    WriteFetchHEAD,
    InvalidFetchHeadEntry,
    FetchHeadFileNotFound,
    InvalidConfigFile,
    InvalidEntryConfigFile,
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
        CommandsError::CreateFetchHEAD => write!(f, "CreateFetchHEAD: No se pudo crear el archivo FETCH_HEAD"),
        CommandsError::ReadFetchHEAD => write!(f, "ReadFetchHEAD: No se pudo leer el archivo FETCH_HEAD"),
        CommandsError::WriteFetchHEAD => write!(f, "WriteFetchHEAD: No se pudo escribir el archivo FETCH_HEAD"),
        CommandsError::InvalidFetchHeadEntry => write!(f, "InvalidFetchHeadEntry: Entrada inválida en FETCH_HEAD"),
        CommandsError::FetchHeadFileNotFound => write!(f, "FetchHeadFileNotFound: No se encontró el archivo FETCH_HEAD"),
        CommandsError::InvalidConfigFile => write!(f, "InvalidConfigFile: Archivo de configuración inválido"),
        CommandsError::InvalidEntryConfigFile => write!(f, "InvalidEntryConfigFile: Entrada inválida en el archivo de configuración"),
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
