use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;

use crate::errors::GitError;

use super::errors::UtilError;

/// Verifica si un directorio está vacío
/// ###Parametros:
/// 'path': url de un directorio
pub fn is_folder_empty(path: &str) -> Result<bool, GitError> {
    println!("{path}");
    let contents = match fs::read_dir(path) {
        Ok(contents) => contents,
        Err(_) => return Err(GitError::VisitDirectoryError),
    };
    let is_empty = contents.count() == 0;

    Ok(is_empty)
}

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
pub fn create_file_replace(file: &str, content: &str) -> Result<(), GitError> {
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

/// Lee un archivo y devuelve el contenido del mismo en String
/// ###Parametros:
/// 'file': archivo a leer.
pub fn read_file_string(mut file: File) -> Result<String, GitError> {
    let mut content = String::new();

    match file.read_to_string(&mut content) {
        Ok(_) => (),
        Err(_) => return Err(GitError::ReadFileError),
    }

    Ok(content)
}

/// Lee un archivo y devuelve el contenido del mismo en un vector
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

/// Elimina un archivo
/// ###Parametros:
/// 'file': ruta del archivo a eliminar.
pub fn delete_file(path_file: &str) -> Result<(),GitError> {
    if fs::metadata(&path_file).is_ok() {
        match fs::remove_file(&path_file) {
            Ok(_) => (),
            Err(_) => return Err(GitError::DeleteFileError),
        }
    } else {
        return Err(GitError::DeleteFileError);
    }
    Ok(())
}

/// Asegura que el directorio esté limpio y, si no existe, lo crea.
///
/// Esta función toma un path de directorio como argumento y utiliza la función auxiliar
/// `_ensure_directory_clean` para realizar la operación. Si ocurriera un error durante
/// la ejecución de `_ensure_directory_clean`, se devuelve un error de tipo `UtilError::CreateDir`.
/// Si la operación se completa correctamente, se devuelve un resultado `Ok(())`.
///
/// # Errores
///
/// - Si `_ensure_directory_clean` devuelve un error al realizar operaciones de E/S
///   (por ejemplo, al crear o eliminar archivos), se convierte en un `UtilError::CreateDir`.
///
/// # Notas
///
/// Esta función se asegura de que el directorio esté completamente limpio, eliminando todos los archivos
/// presentes en él. Úsela con precaución, ya que puede borrar datos no deseados.
///
pub fn ensure_directory_clean(directory: &str) -> Result<(), UtilError> {
    if _ensure_directory_clean(directory).is_err() {
        return Err(UtilError::CreateDir(directory.to_string()));
    }
    Ok(()) // [TESTS]
}

/// Implementación interna que asegura que el directorio esté limpio.
fn _ensure_directory_clean(directory: &str) -> io::Result<()> {
    if !std::path::Path::new(&directory).exists() {
        fs::create_dir_all(&directory)?;
    } else {
        // Elimina todos los archivos existentes en el directorio
        for entry in fs::read_dir(&directory)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                fs::remove_file(&path)?;
            }
        }
    }
    Ok(())
}
