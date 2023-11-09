// use std::fmt;

// use crate::commands::errors::CommandsError;

// pub enum GitServerError
// {
//     VersionNotSentDiscoveryReferences,
//     ServerFromCommands(String),
//     UtilFromCommands(String),
// }

// fn format_error(error: &GitServerError, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//     match error {
//         GitServerError::VersionNotSentDiscoveryReferences => {
//             write!(f, "Error al enviar la versión del protocolo")
//         }
//         GitServerError::UtilFromCommands(info) => write!(f, "{}", info),

//     }
// }

// impl From<CommandsError> for GitServerError {
//     fn from(error: CommandsError) -> Self {
//         GitServerError::ServerFromCommands(format!("{}", error))
//     }
// }

// impl From<GitServerError> for GitError {
//     fn from(err: GitServerError) -> Self {
//         GitError::GitFromUtilError(format!("{}", err))
//     }
// }

// // Esto no se toca
// impl fmt::Display for GitServerError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         format_error(self, f)
//     }
// }

// // Esto no se toca
// impl fmt::Debug for GitServerError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         format_error(self, f)
//     }
// }
