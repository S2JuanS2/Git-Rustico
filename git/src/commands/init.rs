use crate::models::client::Client;
use std::fs;
use std::io::Write;
use std::path::Path;

use crate::errors::GitError;

const INITIAL_BRANCH: &str = "main";

/// Esta función se encarga de llamar al comando init con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función init
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_init(args: Vec<&str>, client: Client) -> Result<String, GitError> {
    if args.len() > 0 {
        return Err(GitError::InvalidArgumentCountInitError);
    }

    let directory = format!("{}", client.get_directory_path());
    let result = git_init(&directory)?;

    Ok(result)
}

/// Crea un directorio si no existe
/// ###Parametros:
/// 'directory': dirección del directorio a crear
fn create_directory(directory: &str) -> Result<(), GitError> {
    if !Path::new(directory).exists() {
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
fn create_file(file: &str, content: &str) -> Result<(), GitError> {
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

/// Esta función inicia un repositorio git creando los directorios y archivos necesarios.
/// ###Parametros:
/// 'directory': dirección donde se inicializará el repositorio.
pub fn git_init(directory: &str) -> Result<String, GitError> {
    create_directory(directory)?;

    let git_dir = format!("{}/.git", directory);
    let objects_dir = format!("{}/objects", &git_dir);
    let heads_dir = format!("{}/refs/heads", &git_dir);

    create_directory(&git_dir)?;
    create_directory(&objects_dir)?;
    create_directory(&heads_dir)?;

    let head_file = format!("{}/HEAD", &git_dir);
    let head_content = format!("ref: /refs/heads/{}\n", INITIAL_BRANCH);
    let index_file = format!("{}/index", &git_dir);

    create_file(&head_file, &head_content)?;
    create_file(&index_file, "")?;

    let result = format!("Repositorio inicializado");

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_git_init() {
        // Se Crea un directorio temporal para el test
        let temp_dir = env::temp_dir().join("test_git_init");
        fs::create_dir(&temp_dir).expect("Falló al crear el directorio temporal");

        // Cuando ejecuto la función git_init en el directorio temporal
        let result = git_init(&temp_dir.to_str().unwrap());

        // No debería resultar en error
        assert!(result.is_ok());

        let git_dir = temp_dir.join(".git");
        assert!(git_dir.exists());

        let objects_dir = git_dir.join("objects");
        assert!(objects_dir.exists());

        let heads_dir = git_dir.join("refs/heads");
        assert!(heads_dir.exists());

        let head_file = git_dir.join("HEAD");
        assert!(head_file.exists());

        fs::remove_dir_all(&temp_dir).expect("Falló al remover el directorio temporal");
    }
}
