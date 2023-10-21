use crate::errors::GitError;

use super::branch::get_branch;

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

const GIT_DIR: &str = "/.git";
const HEAD_FILE: &str = "HEAD";

/// Cambia a otra branch existente
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
/// 'branch_name': Nombre de la branch a cambiar.
pub fn git_checkout_switch(directory: &str, branch_name: &str) -> Result<(), GitError> {
    //Falta implementar que verifique si realizó commit ante la pérdida de datos.
    let branches = get_branch(directory)?;
    if !branches.contains(&branch_name.to_string()) {
        return Err(GitError::BranchDoesntExistError);
    }

    let directory_git = format!("{}{}", directory, GIT_DIR);
    let head_file_path = Path::new(&directory_git).join(HEAD_FILE);

    let mut file = match OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(head_file_path)
    {
        Ok(file) => file,
        Err(_) => return Err(GitError::BranchDirectoryOpenError),
    };

    let content = format!("ref: /refs/heads/{}\n", branch_name);
    if file.write_all(content.as_bytes()).is_err() {
        return Err(GitError::BranchFileWriteError);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::branch::{git_branch_create, git_branch_delete};
    use std::fs;

    const TEST_DIRECTORY: &str = "./test_repo";
    const BRANCH_DIR: &str = "refs/heads/";

    #[test]
    fn test_git_checkout_switch_error() {
        let branch_path = format!("{}{}/{}", TEST_DIRECTORY, GIT_DIR, BRANCH_DIR);
        if let Err(err) = fs::create_dir_all(&branch_path) {
            panic!("Falló al crear el directorio: {}", err);
        }
        // Cuando ejecuto la función
        let result = git_checkout_switch(TEST_DIRECTORY, "test_branch_switch1");
        
        // Limpia el archivo de prueba
        if !Path::new(TEST_DIRECTORY).exists(){
            fs::remove_dir_all(TEST_DIRECTORY).expect("Falló al remover el directorio temporal");
        };

        // Entonces la función lanza error
        assert!(result.is_err());
    }

    #[test]
    fn test_git_checkout_switch_ok() {
        let branch_path = format!("{}{}/{}", TEST_DIRECTORY, GIT_DIR, BRANCH_DIR);
        if let Err(err) = fs::create_dir_all(&branch_path) {
            panic!("Falló al crear el directorio: {}", err);
        }
        let _ = git_branch_delete(TEST_DIRECTORY, "test_branch_switch2");
        git_branch_create(TEST_DIRECTORY, "test_branch_switch2", "commit_hash_branch")
            .expect("Falló en la creación de la branch");
        // Cuando ejecuto la función
        let result = git_checkout_switch(TEST_DIRECTORY, "test_branch_switch2");

        

        // Limpia el archivo de prueba
        if !Path::new(TEST_DIRECTORY).exists(){
            fs::remove_dir_all(TEST_DIRECTORY).expect("Falló al remover el directorio temporal");
        }

        // Entonces la función no lanza error.
        assert!(result.is_ok());
    }
}
