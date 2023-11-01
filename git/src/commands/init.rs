use crate::consts::*;
use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::files::*;
use std::path::Path;

/// Esta función se encarga de llamar al comando init con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función init
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_init(args: Vec<&str>, client: Client) -> Result<String, GitError> {
    if !args.is_empty() {
        return Err(GitError::InvalidArgumentCountInitError);
    }
    let result = git_init(client.get_directory_path())?;

    Ok(result)
}

/// Esta función inicia un repositorio git creando los directorios y archivos necesarios.
/// ###Parametros:
/// 'directory': dirección donde se inicializará el repositorio.
pub fn git_init(directory: &str) -> Result<String, GitError> {
    create_directory(Path::new(directory))?;

    let git_dir = format!("{}/{}", directory, GIT_DIR);
    let objects_dir = format!("{}/{}", &git_dir, DIR_OBJECTS);
    let heads_dir = format!("{}/{}", &git_dir, REF_HEADS);

    create_directory(Path::new(&git_dir))?;
    create_directory(Path::new(&objects_dir))?;
    create_directory(Path::new(&heads_dir))?;

    let head_file = format!("{}/{}", &git_dir, HEAD);
    let head_content = format!("{}{}\n", HEAD_POINTER_REF, INITIAL_BRANCH);
    let index_file = format!("{}/{}", &git_dir, INDEX);

    create_file(&head_file, &head_content)?;
    create_file(&index_file, CONTENT_EMPTY)?;

    let result = format!(
        "Repositorio vacío inicializado en la direcección: {}",
        directory
    );

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_git_init() {
        // Se Crea un directorio temporal para el test
        let temp_dir = Path::new("git_init_test");
        create_directory(temp_dir).expect("Falló al crear el directorio de prueba");

        // Cuando ejecuto la función git_init en el directorio temporal
        let result = git_init("git_init_test");

        // No debería resultar en error
        assert!(result.is_ok());

        let git_dir = temp_dir.join(GIT_DIR);
        assert!(git_dir.exists());

        let objects_dir = git_dir.join(DIR_OBJECTS);
        assert!(objects_dir.exists());

        let heads_dir = git_dir.join(REF_HEADS);
        assert!(heads_dir.exists());

        let head_file = git_dir.join(HEAD);
        assert!(head_file.exists());

        fs::remove_dir_all(&temp_dir).expect("Falló al remover el directorio temporal");
    }
}
