use std::{io, fs};
use std::io::Write;
use std::fs::File;
use std::path::Path;
use std::fs::OpenOptions;

const GIT_DIR: &str = "/.git";
const HEAD_FILE: &str = "HEAD";
const BRANCH_DIR: &str = "refs/heads/";


/// Muestra por pantalla las branch existentes.
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
pub fn git_branch_list(directory: &str) -> io::Result<()>{

    // "directory/.git/refs/heads"
    let directory_git = format!("{}{}",directory,GIT_DIR);
    let branch_dir = Path::new(&directory_git).join(BRANCH_DIR);

    let entries = fs::read_dir(branch_dir)?;

    for entry in entries {
        let entry = entry?;
        println!("{:?}", entry.file_name().into_string());
    }

    Ok(())

}

/// Crea una nueva branch si no existe.
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
/// 'branch_name': Nombre de la branch a crear.
/// 'commit_hash': Contiene el hash del ultimo commit.
pub fn git_branch_create(directory: &str, branch_name: &str, commit_hash: &str) -> io::Result<()>{

    //Falta implementar que busque si ya existe una branch con ese nombre.

    // Crear un nuevo archivo en .git/refs/heads/ con el nombre de la rama y el contenido es el hash del commit actual.
    let branch_path = format!("{}{}/{}{}", directory, GIT_DIR, BRANCH_DIR, branch_name);

    let mut file = File::create(&branch_path)?;
    write!(file, "{}", commit_hash)?;

    Ok(())

}

/// Elimina una branch existente
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
/// 'branch_name': Nombre de la branch a eliminar.
pub fn git_branch_delete(directory: &str, branch_name: &str) -> io::Result<()>{

    // Crear un nuevo archivo en .git/refs/heads/ con el nombre de la rama y el contenido es el hash del commit actual.
    let branch_path = format!("{}{}/{}{}", directory, GIT_DIR, BRANCH_DIR, branch_name);

    fs::remove_file(&branch_path)?;

    Ok(())

}

/// Cambia a otra branch existente
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
/// 'branch_name': Nombre de la branch a cambiar.
pub fn git_branch_switch(directory: &str, branch_name: &str) -> io::Result<()>{

    //Falta implementar que verifique si realizó commit ante la pérdida de datos.
    //Falta implementar que exista la branch a la que desea cambiar.

    let directory_git = format!("{}{}",directory,GIT_DIR);
    let head_file_path = Path::new(&directory_git).join(HEAD_FILE);

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(head_file_path)?;

    let content = format!("ref: /refs/heads/{}\n", branch_name);
    file.write_all(content.as_bytes())?;


    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const TEST_DIRECTORY: &str = "./test_repo";

    #[test]
    fn test_git_branch_list() {
        // Crea una rama ficticia y el directorio
        let branch_name = "test_branch";
        let branch_path = format!("{}{}/{}", TEST_DIRECTORY, GIT_DIR, BRANCH_DIR);

        if let Err(err) = fs::create_dir_all(&branch_path) {
            panic!("Failed to create test directory: {}", err);
        }

        let branch_path_file = format!("{}{}/{}{}", TEST_DIRECTORY, GIT_DIR, BRANCH_DIR, branch_name);
        fs::File::create(&branch_path_file).expect("Failed to create test branch file");

        // Cuando ejecuta la función
        let result = git_branch_list(TEST_DIRECTORY);

        // Entonces la función no lanza error.
        assert!(result.is_ok());

        // Limpia el archivo de prueba
        fs::remove_file(&branch_path_file).expect("Failed to remove test branch file");
        fs::remove_dir_all(branch_path).expect("Failed to remove test branch directory");
    }

    #[test]
    fn test_git_branch_create() {

        let branch_path = format!("{}{}/{}", TEST_DIRECTORY, GIT_DIR, BRANCH_DIR);
        if let Err(err) = fs::create_dir_all(&branch_path) {
            panic!("Failed to create test directory: {}", err);
        }
        // Cuando ejecuto la función
        let result = git_branch_create(TEST_DIRECTORY, "test_new_branch", "commit_hash_branch");

        // Entonces la función no lanza error.
        assert!(result.is_ok());
    }

    #[test]
    fn test_git_branch_delete() {

        let branch_path = format!("{}{}/{}", TEST_DIRECTORY, GIT_DIR, BRANCH_DIR);
        if let Err(err) = fs::create_dir_all(&branch_path) {
            panic!("Failed to create test directory: {}", err);
        }

        // Crea una rama ficticia
        let branch_name = "test_branch_delete";
        let branch_path = format!("{}{}/{}{}", TEST_DIRECTORY, GIT_DIR, BRANCH_DIR, branch_name);
        fs::File::create(&branch_path).expect("Failed to create test branch file");

        // Cuando ejecuto la función
        let result = git_branch_delete(TEST_DIRECTORY, branch_name);

        // Entonces la función no lanza error.
        assert!(result.is_ok());

        // Entonces la rama ha sido eliminada.
        assert!(fs::metadata(&branch_path).is_err());
    }

    #[test]
    fn test_git_branch_switch() {

        let branch_path = format!("{}{}/{}", TEST_DIRECTORY, GIT_DIR, BRANCH_DIR);
        if let Err(err) = fs::create_dir_all(&branch_path) {
            panic!("Failed to create test directory: {}", err);
        }
        // Cuando ejecuto la función
        let result = git_branch_switch(TEST_DIRECTORY, "test_branch_switch");

        // Entonces la función no lanza error.
        assert!(result.is_ok());
    }
}
