use std::fmt;

use gtk::gdk::keys::constants::U;

use crate::{errors::GitError, commands::errors::CommandsError};

pub enum UtilError {
    UtilFromCommands(String), // Para tener polimorfismo con CommandsError
    InvalidPacketLineInfo(String),
    InvalidPacketLine,
    ServerConnection,
    ClientConnection,
}

fn format_error(error: &UtilError, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match error {
        UtilError::UtilFromCommands(info) => write!(f, "{}", info),
        UtilError::InvalidPacketLine => write!(f, "InvalidPacketLineError: Error al leer una línea de paquete."),
        UtilError::InvalidPacketLineInfo(info) => write!(f, "{}\nMore info: {}", UtilError::InvalidPacketLine, info),
        UtilError::ServerConnection => write!(f, "ServerConnectionError: Error al iniciar el servidor."),
        UtilError::ClientConnection => write!(f, "ClientConnectionError: Error al iniciar el cliente."),
        // AGregar más errores aquí
    }
}

// impl From<UtilError> for CommandsError {
//     fn from(err: UtilError) -> Self {
//         CommandsError::CommandsFromUtil(format!("{}", err))
//     }
// }
impl From<CommandsError> for UtilError {
    fn from(error: CommandsError) -> Self {
        UtilError::UtilFromCommands(format!("{}", error))
    }
}

impl From<UtilError> for GitError {
    fn from(err: UtilError) -> Self {
        GitError::GitFromUtilError(format!("{}", err))
    }
}

// Esto no se toca
impl fmt::Display for UtilError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_error(self, f)
    }
}

// Esto no se toca
impl fmt::Debug for UtilError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_error(self, f)
    }
}