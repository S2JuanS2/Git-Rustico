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
    InvalidArgumentCountAddError,
    ReadDirError,
    InvalidArgumentCountError,
    GenericError, // Error genérico, lo uso para tests.
    OpenFileError,
    RemoveFileError,
    ReadFileError,
    CreateFileError,
    WriteFileError,
    CopyFileError,
    CreateDirError,
    ReadBranchesError,
    AlreadyOnThatBranch,
    BranchDirectoryOpenError,
    BranchAlreadyExistsError,
    BranchFileCreationError,
    BranchFileWriteError,
    DeleteBranchError,
    BranchNotFoundError,
    BranchFileReadError,
    HashObjectInvalid,
    RemoteDoesntExistError,
    HeadBranchError,
    VisitDirectoryError,
    GetHashError,
    DirEntryError,
    PathToStringError,
    DirectoryOpenError,
    InvalidArgumentCountBranchError,
    InvalidArgumentCountCatFileError,
    FlagCatFileNotRecognizedError,
    InvalidArgumentCountCheckoutError,
    FlagCheckoutNotRecognisedError,
    InvalidArgumentCountCloneError,
    InvalidArgumentCountCommitError,
    FlagCommitNotRecognizedError,
    InvalidArgumentCountHashObjectError,
    FlagHashObjectNotRecognizedError,
    FlagLsFilesNotRecognizedError,
    InvalidArgumentCountInitError,
    InvalidArgumentCountStatusError,
    InvalidArgumentCountLsFilesError,
    InvalidArgumentCountLsTreeError,
    InvalidTreeHashError,
    InvalidArgumentCountLogError,
    InvalidArgumentCountMergeError,
    InvalidArgumentCountPullError,
    InvalidArgumentCountPushError,
    InvalidArgumentCountRemoteError,
    InvalidArgumentCountRmError,
    InvalidArgumentShowRefError,
    InvalidArgumentCountCheckIgnoreError,
    InvalidSrcDirectoryError,
    RemoteAlreadyExistsError,
    RemoteDoesNotExistError,
    InvalidArgumentCountTagError,
    TagDirectoryOpenError,
    ReadTagsError,
    TagAlreadyExistsError,
    TagNotExistsError,
    InvalidArgumentCountRebaseError,
    BranchNotFound,
    PullCurrentBranchNotFound,
    DeleteReferenceFetchHead,
    ReferenceNotFound,
    InvalidArgumentCountPush,
    RemoteNotFound,
    NoTrackingInformationForBranch,
    MergeNotAllowedError,
    PullRemoteBranchNotFound,
}

fn format_error(error: &CommandsError, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match error {
        CommandsError::CommandsFromUtil(info) => write!(f, "{}", info),
        CommandsError::CloneMissingRepo => {
            write!(f, "CloneMissingRepo: Use: <repositorio>")
        }
        CommandsError::CommitEmptyIndex => write!(f, "Nada al que hacer Commit"),
        CommandsError::InvalidArgumentCountFetchError => {
            write!(f, "InvalidArgumentCountFetchError")
        }
        CommandsError::CloneMissingRepoError => {
            write!(f, "CloneMissingRepoError: Use: <repositorio>")
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
        CommandsError::InvalidArgumentCountAddError => write!(f, "Número de argumentos inválido para el comando add.\nUsar: <file name>"),
        CommandsError::InvalidArgumentCountError => write!(f, "Número de argumentos inválido.\nUse: cargo run -- <path config>"),
        CommandsError::InvalidSrcDirectoryError => write!(f, "Directorio de código fuente inválido, revise su archivo de configuración."),
        CommandsError::GenericError => write!(f, "Error generico."),
        CommandsError::OpenFileError => write!(f, "No se pudo abrir el archivo"),
        CommandsError::RemoveFileError => write!(f, "No se pudo eliminar el archivo"),
        CommandsError::ReadFileError => write!(f, "No se pudo leer el archivo"),
        CommandsError::AlreadyOnThatBranch => write!(f, "Ya estas en esa rama!"),
        CommandsError::ReadBranchesError => write!(f, "No se pudieron leer las branchs del repositorio."),
        CommandsError::BranchDirectoryOpenError => write!(f, "No se pudo abrir el directorio de branchs."),
        CommandsError::BranchAlreadyExistsError => write!(f, "fatal: la rama ya existe"),
        CommandsError::BranchFileCreationError => write!(f, "No se pudo crear el archivo de la branch."),
        CommandsError::BranchFileWriteError => write!(f, "No se pudo escribir en el archivo de la branch."),
        CommandsError::DeleteBranchError => write!(f, "No se pudo borrar la branch"),
        CommandsError::BranchNotFoundError => write!(f, "fatal: la rama no existe"),
        CommandsError::BranchFileReadError => write!(f, "No se pudo leer el archivo de la branch."),
        CommandsError::HashObjectInvalid => write!(f, "Hash del Objeto inválido"),
        CommandsError::CreateFileError => write!(f, "Fallo al crear el archivo"),
        CommandsError::WriteFileError => write!(f, "Fallo al escribir en el archivo"),
        CommandsError::CopyFileError => write!(f, "Fallo al copiar el archivo"),
        CommandsError::CreateDirError => write!(f, "Fallo al crear el directorio"),
        CommandsError::RemoteDoesntExistError => write!(f, "fatal: el repositorio remoto no existe"),
        CommandsError::ReadDirError => write!(f, "Falló al leer el directorio"),
        CommandsError::DirEntryError => write!(f, "Falló al obtener la entrada del directorio"),
        CommandsError::HeadBranchError => write!(f, "No se pudo obtener la rama HEAD"),
        CommandsError::VisitDirectoryError => write!(f, "No se pudo recorrer el directorio"),
        CommandsError::GetHashError => write!(f, "No se pudo obtener el hash del objeto"),
        CommandsError::PathToStringError => write!(f, "No se pudo convertir el path a str"),
        CommandsError::DirectoryOpenError => write!(f, "No se pudo abrir el directorio"),
        CommandsError::InvalidArgumentCountBranchError => write!(f, "Número de argumentos inválido para el comando branch."),
        CommandsError::InvalidArgumentCountCatFileError => write!(f, "Número de argumentos inválido para el comando cat-file.\nUsar: <object hash>"),
        CommandsError::FlagCatFileNotRecognizedError => write!(f, "Flag no reconocida para el comando cat-file"),
        CommandsError::InvalidArgumentCountCheckoutError => write!(f, "Número de argumentos inválido para el comando checkout."),
        CommandsError::FlagCheckoutNotRecognisedError => write!(f, "Flag no reconocida para el comando checkout"),
        CommandsError::InvalidArgumentCountCloneError => write!(f, "Número de argumentos inválido para el comando clone.\nUsar: <url path>"),
        CommandsError::InvalidArgumentCountCommitError => write!(f, "Número de argumentos inválido para el comando commit.\nUsar: <message>"),
        CommandsError::FlagCommitNotRecognizedError => write!(f, "Flag no reconocida para el comando commit"),
        CommandsError::InvalidArgumentCountHashObjectError => write!(f, "Número de argumentos inválido para el comando hash-object.\nUsar: <file name>"),
        CommandsError::FlagHashObjectNotRecognizedError => write!(f, "Flag no reconocida para el comando hash-object"),
        CommandsError::InvalidArgumentCountInitError => write!(f, "Número de argumentos inválido para el comando init.\nUsar: git init"),
        CommandsError::InvalidArgumentCountStatusError => write!(f, "Número de argumentos inválido para el comando status.\n"),
        CommandsError::InvalidArgumentCountLogError => write!(f, "Número de argumentos inválido para el comando log.\n"),
        CommandsError::InvalidArgumentCountMergeError => write!(f, "Número de argumentos inválido para el comando merge.\nUsar: <branch name>"),
        CommandsError::InvalidArgumentCountPullError => write!(f, "Número de argumentos inválido para el comando pull.\nUsar: <branch name>"),
        CommandsError::InvalidArgumentCountPushError => write!(f, "Número de argumentos inválido para el comando push.\nUsar: <branch name>"),
        CommandsError::InvalidArgumentCountRemoteError => write!(f, "Número de argumentos inválido para el comando remote.\nUsar: <flag> <remote name> <url>"),
        CommandsError::InvalidArgumentCountRmError => write!(f, "Número de argumentos inválido para el comando rm.\nUsar: <file name>"),
        CommandsError::InvalidArgumentCountLsFilesError => write!(f, "Número de argumentos inválido para el comando ls-files.\nUsar: <flag>"),
        CommandsError::FlagLsFilesNotRecognizedError => write!(f, "Flag no reconocida para el comando ls-files"),
        CommandsError::InvalidArgumentCountLsTreeError => write!(f, "Número de argumentos inválido para el comando ls-tree.\nUsar: <tree-hash>"),
        CommandsError::InvalidTreeHashError => write!(f, "fatal: not a tree object"),
        CommandsError::InvalidArgumentShowRefError => write!(f, "Número de argumentos inválido para el comando show-ref.\nUsar: git show-ref"),
        CommandsError::InvalidArgumentCountCheckIgnoreError => write!(f, "Número de argumentos inválido para el comando check-ignore.\nUsar: <path name> o --stdin"),
        CommandsError::RemoteAlreadyExistsError => write!(f, "El repositorio remoto ya existe"),
        CommandsError::RemoteDoesNotExistError => write!(f, "El repositorio remoto no existe"),
        CommandsError::InvalidArgumentCountTagError => write!(f, "Número de argumentos inválido para el comando tag.\nUsar: <name_tag> <msg> o <name_tag_delete>"),
        CommandsError::TagDirectoryOpenError => write!(f, "No se pudo abrir el directorio de la tag"),
        CommandsError::ReadTagsError => write!(f, "Error al leer la tag"),
        CommandsError::TagAlreadyExistsError => write!(f, "Ya existe una tag con ese nombre"),
        CommandsError::TagNotExistsError => write!(f, "La tag no existe"),
        CommandsError::InvalidArgumentCountRebaseError => write!(f, "Número de argumentos inválido para el comando rebase.\nUsar: <branch name>"),
        CommandsError::BranchNotFound => write!(f, "La branch no existe"),
        CommandsError::PullCurrentBranchNotFound => write!(f, "Erro al hacer pull, no se pudo obtener la branch actual"),
        CommandsError::DeleteReferenceFetchHead => write!(f, "No se pudo borrar la referencia en FETCH_HEAD"),
        CommandsError::ReferenceNotFound => write!(f, "No se encontró la referencia"),
        // CommandsError::InvalidArgumentCountPush => write!(f, "Número de argumentos inválido para el comando push.\nUsar: git push <remote name> <branch name>"),
        CommandsError::InvalidArgumentCountPush => write!(f, "Número de argumentos inválido para el comando push.\nUsar: git push"),
        CommandsError::RemoteNotFound => write!(f, "No se encontró el repositorio remoto"),
        CommandsError::NoTrackingInformationForBranch => write!(f, "No se encontró información de seguimiento para la branch"),
        CommandsError::MergeNotAllowedError => write!(f, "No se puede hacer merge. La branch no está actualizada con respecto a la branch remota"),
        CommandsError::PullRemoteBranchNotFound => write!(f, "No se encontró la branch remota"),
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
