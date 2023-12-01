use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;

use super::errors::UtilError;
use super::validation::join_paths_correctly;

/// Verifica si un directorio está vacío
/// ###Parametros:
/// 'path': url de un directorio
pub fn is_folder_empty(path: &str) -> Result<bool, UtilError> {
    let contents = match fs::read_dir(path) {
        Ok(contents) => contents,
        Err(_) => return Err(UtilError::VisitDirectoryError),
    };
    let is_empty = contents.count() == 0;

    Ok(is_empty)
}

/// Crea un directorio si no existe
/// ###Parametros:
/// 'directory': dirección del directorio a crear
pub fn create_directory(directory: &Path) -> Result<(), UtilError> {
    if !directory.exists() {
        match fs::create_dir_all(directory) {
            Ok(_) => (),
            Err(_) => return Err(UtilError::CreateDirError),
        };
    }
    Ok(())
}

/// Crea un archivo si no existe.
/// ###Parametros:
/// 'file': archivo a crear.
/// 'content': contenido que se escribirá en el archivo.
pub fn create_file_replace(file: &str, content: &str) -> Result<(), UtilError> {
    let mut file = match fs::File::create(file) {
        Ok(file) => file,
        Err(_) => return Err(UtilError::CreateFileError),
    };
    match file.write_all(content.as_bytes()) {
        Ok(_) => (),
        Err(_) => return Err(UtilError::WriteFileError),
    };

    Ok(())
}

/// Crea un archivo si no existe.
/// ###Parametros:
/// 'file': archivo a crear.
/// 'content': contenido que se escribirá en el archivo.
pub fn create_file(file: &str, content: &str) -> Result<(), UtilError> {
    if fs::metadata(file).is_ok() {
        return Ok(());
    }

    let mut file = match fs::File::create(file) {
        Ok(file) => file,
        Err(_) => return Err(UtilError::CreateFileError),
    };
    match file.write_all(content.as_bytes()) {
        Ok(_) => (),
        Err(_) => return Err(UtilError::WriteFileError),
    };

    Ok(())
}

/// Abre un archivo
/// ###Parametros:
/// 'file': archivo a abrir.
pub fn open_file(file_path: &str) -> Result<File, UtilError> {
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => return Err(UtilError::OpenFileError),
    };

    Ok(file)
}

/// Lee un archivo y devuelve el contenido del mismo en String
/// ###Parametros:
/// 'file': archivo a leer.
pub fn read_file_string(mut file: File) -> Result<String, UtilError> {
    let mut content = String::new();

    match file.read_to_string(&mut content) {
        Ok(_) => (),
        Err(_) => return Err(UtilError::ReadFileError),
    }

    Ok(content)
}

/// Lee un archivo y devuelve el contenido del mismo en un vector
/// ###Parametros:
/// 'file': archivo a leer.
pub fn read_file(mut file: File) -> Result<Vec<u8>, UtilError> {
    let mut content = Vec::new();

    match file.read_to_end(&mut content) {
        Ok(_) => (),
        Err(_) => return Err(UtilError::ReadFileError),
    }

    Ok(content)
}

/// Elimina un archivo
/// ###Parametros:
/// 'file': ruta del archivo a eliminar.
pub fn delete_file(path_file: &str) -> Result<(),UtilError> {
    if fs::metadata(&path_file).is_ok() {
        match fs::remove_file(&path_file) {
            Ok(_) => (),
            Err(_) => return Err(UtilError::DeleteFileError),
        }
    } else {
        return Err(UtilError::DeleteFileError);
    }
    Ok(())
}

/// Verifica si hay un repositorio git en la carpeta (chequea si hay una subcarpeta .git)
/// devuelve una tupla donde el primer elemento es un true o false (según lo anterior) 
/// y el segundo es el nombre del repositorio.
pub fn is_git_initialized(current_src: &str) -> (bool,String) {
    let mut result = (false,"".to_string());
    let path_git = join_paths_correctly(current_src, ".git");
    if fs::read_dir(path_git).is_ok() {
        let parts: Vec<&str> = current_src.split('/').collect();
        let name = parts[parts.len() - 1];
        result = (true,name.to_string());
    }
    result
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
