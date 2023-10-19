#[derive(Debug, PartialEq)]
/// Enumeración que representa los posibles errores que pueden ocurrir durante la ejecución
/// del progbranch Git.
///
/// Cada variante de este enum representa un tipo específico de error que puede ocurrir, y
/// se utiliza para identificar y manejar los errores de manera adecuada.
///
pub enum GitError {
    MissingConfigPathError,
    ConfigFileError,
    InvalidArgumentCountError,
    InvalidConfigurationValueError,
    InvalidUserNameError,
    InvalidUserMailError,
    InvalidPortError,
    InvalidLogDirectoryError,
    InvalidIpError,
    ServerConnectionError,
    ClientConnectionError,
    ObjectBuildFailed,
    GtkFailedInitiliaze,
    OpenFileError,
    ReadFileError,
    ReadBranchesError,
    BranchDirectoryOpenError,
    BranchAlreadyExistsError,
    BranchFileCreationError,
    BranchFileWriteError,
    DeleteBranchError,
    BranchDoesntExistError,
    BranchNotFoundError,
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
            GitError::InvalidConfigurationValueError => "Valor de configuración inválido, revise su archivo de configuración.",
            GitError::InvalidUserNameError => "Nombre de usuario inválido, revise su archivo de configuración.",
            GitError::InvalidUserMailError => "Correo de usuario inválido, revise su archivo de configuración.",
            GitError::InvalidPortError => "Puerto inválido, revise su archivo de configuración.",
            GitError::InvalidLogDirectoryError => "Directorio de logs inválido, revise su archivo de configuración.",
            GitError::InvalidIpError => "Dirección IP inválida, revise su archivo de configuración.",
            GitError::ServerConnectionError => "No se pudo conectar al servidor.",
            GitError::ClientConnectionError => "No se pudo conectar el cliente.",
            GitError::ObjectBuildFailed => "No se pudo obtener el objeto del constructor.",
            GitError::GtkFailedInitiliaze => "No se pudo inicializar GTk",
            GitError::OpenFileError => "No se pudo abrir el archivo",
            GitError::ReadFileError => "No se pudo leer el archivo",
            GitError::ReadBranchesError => "No se pudieron leer las branchs del repositorio.",
            GitError::BranchDirectoryOpenError => "No se pudo abrir el directorio de branchs.",
            GitError::BranchAlreadyExistsError => "fatal: la rama ya existe",
            GitError::BranchFileCreationError => "No se pudo crear el archivo de la branch.",
            GitError::BranchFileWriteError => "No se pudo escribir en el archivo de la branch.",
            GitError::DeleteBranchError => "No se pudo borrar la branch",
            GitError::BranchDoesntExistError => "Ruta especificada no concordó con ningún archivo conocido por git",
            GitError::BranchNotFoundError => "fatal: la rama no existe",
        }
    }
}
