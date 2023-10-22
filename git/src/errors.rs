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
    InvalidConfigFormatError,
    InvalidArgumentCountError,
    InvalidConfigurationValueError,
    InvalidUserNameError,
    InvalidUserMailError,
    InvalidPortError,
    InvalidLogDirectoryError,
    InvalidIpError,
    ServerConnectionError,
    ClientConnectionError,
    InvalidPacketLineError,
    InvalidServerReferenceError,
    InvalidVersionNumberError,
    InvalidObjectIdError,
    GenericError, // Error genérico, lo uso para tests.
    UploadRequest,
    PackfileNegotiationNACK,
    ObjectBuildFailed,
    GtkFailedInitiliaze,
    OpenFileError,
    ReadFileError,
    CreateFileError,
    WriteFileError,
    CopyFileError,
    CreateDirError,
    ReadBranchesError,
    BranchDirectoryOpenError,
    BranchAlreadyExistsError,
    BranchFileCreationError,
    BranchFileWriteError,
    DeleteBranchError,
    BranchDoesntExistError,
    BranchNotFoundError,
    HashObjectInvalid,
    DecompressionFailed,
    RemoteDoesntExistError,
    WriteStreamError,
    SendCommandError,
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
            GitError::InvalidLogDirectoryError => "Directorio de logs inválido, revise su archivo de configuración.",
            GitError::InvalidIpError => "Dirección IP inválida, revise su archivo de configuración.",
            GitError::ServerConnectionError => "No se pudo conectar al servidor.",
            GitError::ClientConnectionError => "No se pudo conectar el cliente.",
            GitError::GenericError => "Error genérico.",
            GitError::InvalidPacketLineError => "Error al leer una línea de paquete.",
            GitError::InvalidVersionNumberError => "Número de versión inválido, solo se acepta v1 y v2.",
            GitError::InvalidServerReferenceError => "Referencia no reconocida por el servidor.",
            GitError::InvalidObjectIdError => "Se encontro un object id no valido del servidor.",
            GitError::UploadRequest => "NO se pudo completar el pedido para la negociacion con el servidor",
            GitError::PackfileNegotiationNACK => "La negocion fallo porque no se recibio el NACK del servidor",
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
            GitError::HashObjectInvalid => "Hash del Objeto inválido",
            GitError::DecompressionFailed => "Falló al descomprimir el archivo",
            GitError::CreateFileError => "Falló al crear el archivo",
            GitError::WriteFileError => "Falló al escribir en el archivo",
            GitError::CopyFileError => "Falló al copiar el archivo",
            GitError::CreateDirError => "Falló al crear el directorio",
            GitError::RemoteDoesntExistError => "fatal: el repositorio remoto no existe",
            GitError::WriteStreamError => "Falló al enviar datos al socket",
            GitError::SendCommandError => "Falló al enviar el comando",
        }
    }
}
