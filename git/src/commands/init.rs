use std::fs;
use std::io::{self, Write};
use std::path::Path;

const INITIAL_BRANCH: &str = "main";

/// Crea un directorio si no existe
/// ###Parametros:
/// 'directory': dirección del directorio a crear
fn create_directory(directory: &str) -> io::Result<()> {
    if !Path::new(directory).exists() {
        fs::create_dir_all(directory)?;
    }
    Ok(())
}

/// Crea un archivo si no existe.
/// ###Parametros:
/// 'file': archivo a crear.
/// 'content': contenido que se escribirá en el archivo.
fn create_file(file: &str, content: &str) -> io::Result<()> {
    if fs::metadata(file).is_ok() {
        return Ok(());
    }

    let mut file = fs::File::create(file)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

/// Esta función inicia un repositorio git creando los directorios y archivos necesarios.
/// ###Parametros:
/// 'directory': dirección donde se inicializará el repositorio.
pub fn git_init(directory: &str) -> io::Result<()> {
    create_directory(directory)?;

    let git_dir = format!("{}/.git", directory);
    let objects_dir = format!("{}/objects", &git_dir);
    let heads_dir = format!("{}/refs/heads", &git_dir);

    create_directory(&git_dir)?;
    create_directory(&objects_dir)?;
    create_directory(&heads_dir)?;

    let head_file = format!("{}/HEAD", &git_dir);
    let head_content = format!("ref: /refs/heads/{}\n", INITIAL_BRANCH);

    create_file(&head_file, &head_content)?;

    println!("Repositorio inicializado en {}", directory);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_git_init() {
        // Se Crea un directorio temporal para el test
        let temp_dir = env::temp_dir().join("test_git_init");
        fs::create_dir(&temp_dir).expect("Failed to create temporary directory");

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

        fs::remove_dir_all(&temp_dir).expect("Failed to remove temporary directory");
    }
}
