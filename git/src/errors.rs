#[derive(Debug, PartialEq)]
/// Enumeración que representa los posibles errores que pueden ocurrir durante la ejecución
/// del programa Git.
///
/// Cada variante de este enum representa un tipo específico de error que puede ocurrir, y
/// se utiliza para identificar y manejar los errores de manera adecuada.
///
pub enum GitError {
    MissingConfigPathError,
    ConfigFileError,
    InvalidConfigFormatError,
    InvalidArgumentCountError,
    InvalidConfigurationValueError,
    InvalidUserNameError,
    InvalidUserMailError,
    InvalidPortError,
    InvalidLogDirectoryError,
    InvalidIpError,
    GenericError, // Error genérico, lo uso para tests.
    ObjectBuildFailed,
    GtkFailedInitiliaze,
    OpenFileError,
    RemoveFileError,
    ReadFileError,
    CreateFileError,
    WriteFileError,
    CopyFileError,
    CreateDirError,
    NonGitCommandError,
    CommandNotRecognizedError,
    WriteStreamError,
    SendCommandError,
    HeaderPackFileReadError,
    GitFromUtilError(String),
    ViewError(String),
    GitFromModelsError(String),
    GitFromControllerError(String),
    GitFromCommandsError(String),
    InvalidSrcDirectoryError,
    GitServerError(String),
    GitFromServerError(String),
    ReadDirError,
    DirEntryError,
    NotAGitRepository,
}

impl GitError {
    /// Obtiene el mensaje descriptivo correspondiente al error actual.
    ///
    /// Esta función devuelve un mensaje de error descriptivo basado en el tipo de error que
    /// se ha producido. Los mensajes de error proporcionados son informativos y ayudan a
    /// identificar la causa del error.
    ///
    /// # Return
    ///
    /// Un valor `&str` que contiene el mensaje descriptivo del error actual.
    ///
    pub fn message(&self) -> &str {
        match self {
            GitError::ConfigFileError => "No se pudo abrir el archivo de configuración.",
            GitError::MissingConfigPathError => "No se ha especificado la ruta del archivo de configuración.\nUse: cargo run --bin <mode> -- <path config>",
            GitError::InvalidArgumentCountError => "Número de argumentos inválido.\nUse: cargo run -- <path config>",
            GitError::InvalidConfigFormatError => "Formato de archivo de configuración inválido. Format: key=value",
            GitError::InvalidConfigurationValueError => "Valor de configuración inválido, revise su archivo de configuración.",
            GitError::InvalidUserNameError => "Nombre de usuario inválido, revise su archivo de configuración.",
            GitError::InvalidUserMailError => "Correo de usuario inválido, revise su archivo de configuración.",
            GitError::InvalidPortError => "Puerto inválido, revise su archivo de configuración.",
            GitError::InvalidSrcDirectoryError => "Directorio de código fuente inválido, revise su archivo de configuración.",
            GitError::InvalidLogDirectoryError => "Path de log inválido, revise su archivo de configuración.",
            GitError::InvalidIpError => "Dirección IP inválida, revise su archivo de configuración.",
            GitError::GenericError => "Error generico.",
            GitError::ObjectBuildFailed => "No se pudo obtener el objeto del constructor.",
            GitError::GtkFailedInitiliaze => "No se pudo inicializar GTk",
            GitError::OpenFileError => "GitError: No se pudo abrir el archivo",
            GitError::RemoveFileError => "No se pudo eliminar el archivo",
            GitError::ReadFileError => "No se pudo leer el archivo",
            GitError::CreateFileError => "Fallo al crear el archivo",
            GitError::WriteFileError => "Fallo al escribir en el archivo",
            GitError::CopyFileError => "Fallo al copiar el archivo",
            GitError::CreateDirError => "Fallo al crear el directorio",
            GitError::WriteStreamError => "Falló al enviar datos al socket",
            GitError::SendCommandError => "Falló al enviar el comando",
            GitError::HeaderPackFileReadError => "Falló al leer el header del packfile recibo del servidor",
            GitError::NonGitCommandError => "Solo se aceptan comandos git. Usage: git <command> -options",
            GitError::CommandNotRecognizedError => "El comando no es reconocido por git",
            GitError::GitFromUtilError(msg) => msg,
            GitError::ViewError(msg) => msg,
            GitError::GitFromModelsError(msg) => msg,
            GitError::GitFromControllerError(msg) => msg,
            GitError::GitFromCommandsError(msg) => msg,
            GitError::GitServerError(msg) => msg,
            GitError::GitFromServerError(msg) => msg,
            GitError::ReadDirError => "Falló al leer el directorio",
            GitError::DirEntryError => "Falló al obtener la entrada del directorio",
            GitError::NotAGitRepository => "not a git repository",
        }
    }
}
