use std::{
    fs::{File, OpenOptions},
    io::{self, Write},
};

use crate::errors::GitError;

/// Enumera los posibles tipos de salida de registro para el programa.
///
/// - `LogFile`: Representa un archivo de registro que almacena los datos de registro.
/// - `StandardError`: Representa el canal de salida estándar de error (stderr) para mensajes de registro.
///
pub enum LogOutput {
    LogFile(File),
    StandardError(io::Stderr),
}

impl LogOutput {
    /// Crea una nueva instancia de LogOutput según el archivo de registro especificado.
    ///
    /// # Arguments
    ///
    /// * `path` - La ruta del archivo de registro a abrir o crear.
    ///
    /// # Returns
    ///
    /// Devuelve un `Result` con `LogOutput` que representa la salida del archivo de registro.
    /// Si el archivo no se pudo abrir devolvera un `LogOutput` que representa la salida estándar de error.
    pub fn new(path: &str) -> Result<LogOutput, GitError> {
        match open_log_file(path) {
            Ok(file) => Ok(LogOutput::LogFile(file)),
            Err(_) => Ok(LogOutput::StandardError(io::stderr())),
        }
    }

    /// Sincroniza todos los datos del archivo de registro o la salida estándar.
    ///
    /// # Returns
    ///
    /// Devuelve un `Result` sin valor si la operación se realizó con éxito,
    /// o un error `GitError` si la sincronización falla.
    /// Solo la sincronización del archivo de registro puede fallar si es del tipo `LogFile``.
    pub fn sync_all(&mut self) -> Result<(), GitError> {
        match self {
            LogOutput::LogFile(file) => {
                if file.sync_all().is_err() {
                    return Err(GitError::LogOutputSyncError);
                };
                Ok(())
            }
            LogOutput::StandardError(_) => Ok(()),
        }
    }
}

/// Abre o crea un archivo de registro en la ruta especificada.
///
/// # Arguments
///
/// * `log_path` - La ruta del archivo de registro a abrir o crear.
///
/// # Returns
///
/// Devuelve un `Result` con el archivo si la operación fue exitosa o un error `GitError`.
fn open_log_file(log_path: &str) -> Result<File, GitError> {
    match OpenOptions::new().create(true).append(true).open(log_path) {
        Ok(file) => Ok(file),
        Err(_) => return Err(GitError::LogOutputOpenError),
    }
}

impl Write for LogOutput {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            LogOutput::LogFile(file) => file.write(buf),
            LogOutput::StandardError(stderr) => stderr.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            LogOutput::LogFile(file) => file.flush(),
            LogOutput::StandardError(stderr) => stderr.flush(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_existing_log_file() {
        let log_path = "../../../README.md";
        let result = LogOutput::new(log_path);
        match result {
            Ok(LogOutput::LogFile(_)) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_new_with_non_existing_log_path() {
        let log_path = "./%&(%(&%/non_existing_log.txt";
        let result = LogOutput::new(log_path);
        match result {
            Ok(LogOutput::StandardError(_)) => assert!(true, "Expected a standard error output."),
            _ => assert!(false, "Expected a standard error output."),
        }
    }

    #[test]
    fn test_open_existing_log_file() {
        let log_path = "../../../README.md";

        // Prueba para abrir un archivo de registro existente
        match open_log_file(log_path) {
            Ok(_) => assert!(true, "Expected a successful file open."),
            Err(e) => assert!(false, "Unexpected error: {}", e.message()),
        }
    }

    #[test]
    fn test_open_non_existing_log_path() {
        let log_path_non_existing = "./%&(%(&%/non_existing_log.txt";
        let result = open_log_file(log_path_non_existing);
        assert!(
            result.is_err(),
            "Expected an error opening a non-existing file."
        );
    }
}
