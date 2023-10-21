use std::fs;
use std::io;
use std::path::Path;

use crate::errors::GitError;

const GIT_DIR: &str = "/.git";
const REMOTES_DIR: &str = "refs/remotes/";

/// Recupera las referencias y objetos del repositorio remoto.
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
/// 'remote_name': nombre del repositorio remoto.
pub fn git_fetch(directory: &str, remote_name: &str) -> Result<(), GitError> {
    // Verifica si el repositorio remoto existe
    let remote_dir = format!("{}{}", REMOTES_DIR, remote_name);
    let remote_refs_dir = format!("{}{}", directory, remote_dir);

    if !Path::new(&remote_refs_dir).exists() {
        return Err(GitError::RemoteDoesntExistError,
        );
    }

    // Copia las referencias del repositorio remoto al directorio local
    let local_refs_dir = format!("{}{}", directory, GIT_DIR);
    let local_refs_dir = Path::new(&local_refs_dir).join("refs/remotes").join(remote_name);

    fs::create_dir_all(&local_refs_dir)?;

    for entry in fs::read_dir(&remote_refs_dir)? {
        if let Ok(entry) = entry {
            let file_name = entry.file_name();
            let local_ref_path = local_refs_dir.join(file_name);
            let remote_ref_path = entry.path();

            fs::copy(remote_ref_path, local_ref_path)?;
        }
    }

    // Descarga los objetos necesarios desde el repositorio remoto
    let objects_dir = format!("{}{}", directory, GIT_DIR);
    for entry in fs::read_dir(&objects_dir)? {
        if let Ok(entry) = entry {
            let file_name = entry.file_name();
            let object_hash = file_name.to_string_lossy().to_string();

            // Usar git_cat_file para descargar el objeto
        }
    }

    Ok(())
}

/*#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_git_fetch() {
        // Se Crea un directorio temporal para el test
        let temp_dir = env::temp_dir().join("test_git_fetch");
        fs::create_dir(&temp_dir).unwrap();

        // Se crea un repositorio remoto
        let remote_dir = temp_dir.join("remote");
        fs::create_dir(&remote_dir).unwrap();
        git_init(&remote_dir.to_string_lossy().to_string()).unwrap();

        // Se crea un repositorio local
        let local_dir = temp_dir.join("local");
        fs::create_dir(&local_dir).unwrap();
        git_init(&local_dir.to_string_lossy().to_string()).unwrap();

        // Se crea un archivo en el repositorio remoto
        let file_path = remote_dir.join("test.txt");
        fs::write(&file_path, "test").unwrap();
        //add  y commit

        // Se agrega el repositorio remoto al repositorio local

        // Se ejecuta el comando fetch
        git_fetch(&local_dir.to_string_lossy().to_string(), "origin").unwrap();

        // Se verifica que el archivo exista en el repositorio local
        let file_path = local_dir.join("test.txt");
        assert!(file_path.exists());

        // Se elimina el directorio temporal
        fs::remove_dir_all(&temp_dir).unwrap();
    }
    
}*/