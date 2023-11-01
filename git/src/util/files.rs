use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;

use crate::errors::GitError;
/// Crea un directorio si no existe
/// ###Parametros:
/// 'directory': dirección del directorio a crear
pub fn create_directory(directory: &Path) -> Result<(), GitError> {
    if !directory.exists() {
        match fs::create_dir_all(directory) {
            Ok(_) => (),
            Err(_) => return Err(GitError::CreateDirError),
        };
    }
    Ok(())
}

/// Crea un archivo si no existe.
/// ###Parametros:
/// 'file': archivo a crear.
/// 'content': contenido que se escribirá en el archivo.
pub fn create_file(file: &str, content: &str) -> Result<(), GitError> {
    if fs::metadata(file).is_ok() {
        return Ok(());
    }

    let mut file = match fs::File::create(file) {
        Ok(file) => file,
        Err(_) => return Err(GitError::CreateFileError),
    };
    match file.write_all(content.as_bytes()) {
        Ok(_) => (),
        Err(_) => return Err(GitError::WriteFileError),
    };

    Ok(())
}

/// Abre un archivo
/// ###Parametros:
/// 'file': archivo a abrir.
pub fn open_file(file_path: &str) -> Result<File, GitError> {
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => return Err(GitError::OpenFileError),
    };

    Ok(file)
}

/// Lee un archivo y devuelve el contenido del mismo
/// ###Parametros:
/// 'file': archivo a leer.
pub fn read_file(mut file: File) -> Result<Vec<u8>, GitError> {
    let mut content = Vec::new();

    match file.read_to_end(&mut content) {
        Ok(_) => (),
        Err(_) => return Err(GitError::ReadFileError),
    }

    Ok(content)
}
