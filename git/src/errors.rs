#[derive(Debug)]
/// Enumeración que representa los posibles errores que pueden ocurrir durante la ejecución
/// del programa Git.
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
    InvalidLogDirectoryError,
    InvalidIpError,
    ServerConnectionError,
    ClientConnectionError,
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
            GitError::MissingConfigPathError => "No se ha especificado la ruta del archivo de configuración.\nUse: cargo run -- <path config>",
            GitError::InvalidArgumentCountError => "Número de argumentos inválido.\nUse: cargo run -- <path config>",
            GitError::InvalidConfigurationValueError => "Valor de configuración inválido, revise su archivo de configuración.",
            GitError::InvalidUserNameError => "Nombre de usuario inválido, revise su archivo de configuración.",
            GitError::InvalidUserMailError => "Correo de usuario inválido, revise su archivo de configuración.",
            GitError::InvalidLogDirectoryError => "Directorio de logs inválido, revise su archivo de configuración.",
            GitError::InvalidIpError => "Dirección IP inválida, revise su archivo de configuración.",
            GitError::ServerConnectionError => "No se pudo conectar al servidor.",
            GitError::ClientConnectionError => "No se pudo conectar el cliente.",
        }
    }
}
