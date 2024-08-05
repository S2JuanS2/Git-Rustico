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

/// Crea un archivo  exista o no.
/// ###Parametros:
/// 'file': archivo a crear.
/// 'content': contenido que se escribirá en el archivo.
pub fn create_file_replace(file: &str, content: &str) -> Result<(), UtilError> {
    let path = Path::new(file);

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            match fs::create_dir_all(parent) {
                Ok(_) => (),
                Err(_) => return Err(UtilError::CreateDirError),
            }
        }
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
    if fs::metadata(path_file).is_ok() {
        match fs::remove_file(path_file) {
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
        fs::create_dir_all(directory)?;
    } else {
        // Elimina todos los archivos existentes en el directorio
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                fs::remove_file(&path)?;
            }
        }
    }
    Ok(())
}

/// Verifica si una carpeta existe dado un path.
///
/// # Argumentos
///
/// * `folder_path` - El path de la carpeta que se desea verificar.
///
/// # Retorna
///
/// `true` si la carpeta existe, `false` en caso contrario.
///
pub fn folder_exists(folder_path: &str) -> bool {
    let path = Path::new(folder_path);
    path.is_dir()
}


/// Verifica si un archivo existe dado un path.
///
/// # Argumentos
///
/// * `file_path` - El path del archivo que se desea verificar.
///
/// # Retorna
///
/// `true` si el archivo existe, `false` en caso contrario.
///
pub fn file_exists(file_path: &str) -> bool {
    let path = Path::new(file_path);
    path.is_file()
}

/// Lista todos los archivos y carpetas en un directorio especificado.
///
/// Esta función toma la ruta de un directorio como parámetro y devuelve un `Vec<String>`
/// con los nombres de todos los archivos y carpetas contenidos en ese directorio.
///
/// # Argumentos
///
/// * `path` - Una referencia a un string slice que contiene la ruta del directorio a listar.
///
/// # Retornos
///
/// Esta función devuelve un `io::Result<Vec<String>>`. Si el directorio se lista correctamente,
/// devuelve un vector de strings con los nombres de los archivos y carpetas. Si ocurre un error,
/// devuelve un `io::Error`.
///
/// # Errores
///
/// Esta función retornará un `io::Error` si:
/// - La ruta especificada no es un directorio.
/// - Ocurre algún error al leer el contenido del directorio.
/// 
pub fn list_directory_contents(path: &str) -> Result<Vec<String>, UtilError> {
    let mut entries = Vec::new();
    let path = Path::new(path);

    if path.is_dir() {
        let dir_enty = match fs::read_dir(path)
        {
            Ok(dir_enty) => dir_enty,
            Err(_) => return Err(UtilError::ReadDirError),
        };
        for entry in dir_enty {
            let entry = match entry
            {
                Ok(entry) => entry,
                Err(_) => return Err(UtilError::ReadDirError),
            };
            let entry_path = entry.path();
            if let Some(entry_name) = entry_path.file_name() {
                if let Some(name_str) = entry_name.to_str() {
                    entries.push(name_str.to_string());
                }
            }
        }
    } else {
        return Err(UtilError::NotDirectory);
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_folder_exists() {
        assert_eq!(folder_exists("./bin"), true);
    }

    #[test]
    fn test_folder_not_exists() {
        assert_eq!(folder_exists("/nonexistent_folder"), false);
    }

    #[test]
    fn test_file_exists() {
        assert_eq!(file_exists("./Cargo.toml"), true); 
    }

    #[test]
    fn test_file_not_exists() {
        assert_eq!(file_exists("/nonexistent_file"), false);
    }
}