use std::fs;
use std::path::Path;

use crate::errors::GitError;

use super::cat_file::git_cat_file;
use crate::models::client::Client;

const GIT_DIR: &str = "/.git";
const REMOTES_DIR: &str = "refs/remotes/";

/// Esta función se encarga de llamar al comando fetch con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función fetch
/// 'client': cliente que contiene el directorio del repositorio local
pub fn handle_fetch(args: Vec<&str>, client: Client) -> Result<(), GitError> {
    // Verifica que se haya ingresado un nombre de repositorio remoto
    let directory = client.get_directory_path();
    if args.len() == 1 {
        git_fetch(&directory, args[0])?;
    } else if args.len() == 2 {
        //fetch para una rama especifica
    } else {
        return Err(GitError::InvalidArgumentCountFetchError);
    }

    Ok(())
}

/// Recupera las referencias y objetos del repositorio remoto.
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
/// 'remote_name': nombre del repositorio remoto.
pub fn git_fetch(directory: &str, remote_name: &str) -> Result<(), GitError> {
    // Verifica si el repositorio remoto existe
    let remote_dir = format!("{}{}", REMOTES_DIR, remote_name);
    let remote_refs_dir = format!("{}{}", directory, remote_dir);

    if !Path::new(&remote_refs_dir).exists() {
        return Err(GitError::RemoteDoesntExistError);
    }

    // Copia las referencias del repositorio remoto al directorio local
    let local_refs_dir = format!("{}{}", directory, GIT_DIR);
    let local_refs_dir = Path::new(&local_refs_dir)
        .join("refs/remotes")
        .join(remote_name);

    if fs::create_dir_all(&local_refs_dir).is_err() {
        return Err(GitError::OpenFileError);
    }

    let entries = match fs::read_dir(&remote_refs_dir) {
        Ok(entries) => entries,
        Err(_) => return Err(GitError::ReadFileError),
    };

    for entry in entries {
        match entry {
            Ok(entry) => {
                let file_name = entry.file_name();
                let local_ref_path = local_refs_dir.join(file_name);
                let remote_ref_path = entry.path();

                if fs::copy(remote_ref_path, local_ref_path).is_err() {
                    return Err(GitError::CopyFileError);
                }
            }
            Err(_) => {
                return Err(GitError::ReadFileError);
            }
        }
    }

    // Descarga los objetos necesarios desde el repositorio remoto
    let objects_dir = format!("{}{}", directory, GIT_DIR);

    let objects = match fs::read_dir(&objects_dir) {
        Ok(objects) => objects,
        Err(_) => return Err(GitError::ReadFileError),
    };

    for entry in objects {
        match entry {
            Ok(entry) => {
                let file_name = entry.file_name();
                let object_hash = file_name.to_string_lossy().to_string();

                git_cat_file(directory, &object_hash)?;
            }
            Err(_) => {
                return Err(GitError::ReadFileError);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_git_fetch() {
        let directory = env::current_dir().unwrap();
        let directory = directory.to_str().unwrap();

        // Crear un repositorio remoto
        let remote_name = "origin";
        let remote_dir = format!("{}{}", REMOTES_DIR, remote_name);
        let remote_refs_dir = format!("{}{}", directory, remote_dir);
        fs::create_dir_all(&remote_refs_dir).unwrap();

        // Crear una referencia en el repositorio remoto
        let remote_ref_path = remote_refs_dir.clone() + "/master";
        let mut file = File::create(remote_ref_path).unwrap();
        let _ = write!(file, "1234567890").unwrap();

        // Crear un objeto en el repositorio remoto
        let objects_dir = format!("{}{}", directory, GIT_DIR);
        let object_dir = objects_dir.clone() + "/12";
        fs::create_dir_all(&object_dir).unwrap();
        let object_path = object_dir + "/34567890";
        let mut file = File::create(object_path).unwrap();
        let _ = write!(file, "test").unwrap();

        // Ejecutar git_fetch
        let result = git_fetch(directory, remote_name);
        println!("{:?}", result);

        // Verificar que la referencia se copió al repositorio local
        let local_refs_dir = format!("{}{}", directory, GIT_DIR);
        let local_refs_dir = Path::new(&local_refs_dir)
            .join("refs/remotes")
            .join(remote_name);
        let local_ref_path = local_refs_dir.join("master");
        assert!(local_ref_path.exists());

        // Verificar que el objeto se copió al repositorio local
        let object_path = objects_dir + "/12/34567890";
        assert!(Path::new(&object_path).exists());

        // Eliminar el repositorio remoto
        if !Path::new(&remote_refs_dir).exists() {
            fs::remove_dir_all(&remote_refs_dir).unwrap();
        }

        // Eliminar el repositorio local
        let local_refs_dir = format!("{}{}", directory, GIT_DIR);
        let local_refs_dir = Path::new(&local_refs_dir)
            .join("refs/remotes")
            .join(remote_name);
        let local_ref_path = local_refs_dir.join("master");
        println!("{:?}", local_ref_path);
        println!("{:?}", local_refs_dir);
        fs::remove_file(&local_ref_path).unwrap();
        fs::remove_dir_all(&local_refs_dir).unwrap();
    }
}
