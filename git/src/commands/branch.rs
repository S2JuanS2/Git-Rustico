use crate::consts::*;
use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::files::{open_file, read_file, read_file_string, create_file};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Esta función se encarga de llamar a al comando branch con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de Strings que contiene los argumentos que se le pasaran al comando branch
/// 'client': Cliente que contiene el directorio del repositorio local
pub fn handle_branch(args: Vec<&str>, client: Client) -> Result<String, GitError> {
    let directory = client.get_directory_path();
    if args.is_empty() {
        git_branch_list(directory)
    } else if args.len() == 1 {
        git_branch_create(directory, args[0])
    } else if (args.len() == 2 && args[0] == "-d") || (args.len() == 2 && args[0] == "-D") {
        git_branch_delete(directory, args[1])
    } else {
        return Err(GitError::InvalidArgumentCountBranchError);
    }
}

pub fn get_current_branch(directory: &str) -> Result<String, GitError> {
    let head_path = format!("{}/{}/HEAD", directory, GIT_DIR);
    let head_file = match File::open(head_path) {
        Ok(file) => file,
        Err(_) => return Err(GitError::BranchDirectoryOpenError),
    };

    let reader = BufReader::new(head_file);
    let mut branch = String::new();
    for line in reader.lines() {
        let line = match line {
            Ok(line) => line,
            Err(_) => return Err(GitError::BranchFileReadError),
        };
        let line_split: Vec<&str> = line.split('/').collect();
        branch = line_split[line_split.len() - 1].to_string();
    }

    Ok(branch)
}

/// Muestra por pantalla las branch existentes.
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
pub fn git_branch_list(directory: &str) -> Result<String, GitError> {
    let branches = get_branch(directory)?;
    let current_branch = get_current_branch(directory)?;
    let mut formatted_branches = String::new();
    for branch in branches {
        if branch == current_branch {
            formatted_branches.push_str(&format!(" *- {}\n", branch))
        } else {
            formatted_branches.push_str(&format!(" - {}\n", branch))
        }
    }

    Ok(formatted_branches)
}

pub fn copy_log(directory: &str, current_branch: &str, branch_name: &str) -> Result<(),GitError>{
    let current_branch_log_path = format!("{}/{}/logs/refs/heads/{}", directory, GIT_DIR, current_branch);
    let new_branch_log_path = format!("{}/{}/logs/refs/heads/{}", directory, GIT_DIR, branch_name);
    let file_log_branch = open_file(&current_branch_log_path)?;
    let content_log_current_branch = read_file_string(file_log_branch)?;
    create_file(&new_branch_log_path, &content_log_current_branch)?;

    Ok(())
}

/// Crea una nueva branch si no existe.
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
/// 'branch_name': Nombre de la branch a crear.
/// 'commit_hash': Contiene el hash del ultimo commit.
pub fn git_branch_create(directory: &str, branch_name: &str) -> Result<String, GitError> {
    let branches = get_branch(directory)?;
    if branches.contains(&branch_name.to_string()) {
        return Err(GitError::BranchAlreadyExistsError);
    }
    let current_branch = get_current_branch(directory)?;
    let branch_current_path = format!("{}/{}/{}/{}", directory, GIT_DIR, REF_HEADS, current_branch);
    if fs::metadata(&branch_current_path).is_err() {
        return Err(GitError::BranchDoesntExistError);
    }
    let file_current_branch = open_file(&branch_current_path)?;
    let hash_current_branch = read_file(file_current_branch)?;

    let commit_current_branch = match String::from_utf8(hash_current_branch) {
        Ok(commit_current_branch) => commit_current_branch,
        Err(_) => return Err(GitError::GenericError),
    };
    // Crear un nuevo archivo en .git/refs/heads/ con el nombre de la rama y el contenido es el hash del commit actual.
    let branch_path = format!("{}/{}/{}/{}", directory, GIT_DIR, REF_HEADS, branch_name);

    let mut file = match File::create(branch_path) {
        Ok(file) => file,
        Err(_) => return Err(GitError::BranchDirectoryOpenError),
    };
    match write!(file, "{}", commit_current_branch) {
        Ok(_) => (),
        Err(_) => return Err(GitError::BranchFileWriteError),
    }
    copy_log(directory, &current_branch, branch_name)?;

    let result = format!("Rama {} creada con éxito!", branch_name);

    Ok(result)
}

// Devuelve un vector con los nombres de las branchs
pub fn get_branch(directory: &str) -> Result<Vec<String>, GitError> {
    // "directory/.git/refs/heads"
    let directory_git = format!("{}/{}", directory, GIT_DIR);
    let branch_dir = Path::new(&directory_git).join(REF_HEADS);

    let entries = match fs::read_dir(branch_dir) {
        Ok(entries) => entries,
        Err(_) => return Err(GitError::BranchDirectoryOpenError),
    };

    let mut branches: Vec<String> = Vec::new();

    for entry in entries {
        match entry {
            Ok(entry) => {
                let branch = match entry.file_name().into_string() {
                    Ok(branch) => branch,
                    Err(_) => return Err(GitError::ReadBranchesError),
                };
                branches.push(branch);
            }
            Err(_) => return Err(GitError::ReadBranchesError),
        }
    }

    Ok(branches)
}

/// Elimina una branch existente
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
/// 'branch_name': Nombre de la branch a eliminar.
pub fn git_branch_delete(directory: &str, branch_name: &str) -> Result<String, GitError> {
    if get_current_branch(directory) == Ok(branch_name.to_string()) {
        return Err(GitError::DeleteBranchError);
    }

    let branches = get_branch(directory)?;
    if !branches.contains(&branch_name.to_string()) {
        return Err(GitError::BranchNotFoundError);
    }

    // Crear un nuevo archivo en .git/refs/heads/ con el nombre de la rama y el contenido es el hash del commit actual.
    let branch_path = format!("{}/{}/{}/{}", directory, GIT_DIR, REF_HEADS, branch_name);

    if fs::remove_file(branch_path).is_err() {
        return Err(GitError::DeleteBranchError);
    }

    Ok("Rama eliminada con éxito".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::checkout::git_checkout_switch;
    use crate::commands::init::git_init;
    use std::fs;
    use std::path::Path;

    const TEST_DIRECTORY: &str = "./test_repo";

    #[test]
    fn test_git_branch_list() {
        // Crea una rama ficticia y el directorio
        let branch_name = "test_branch";
        let branch_path = format!("{}/{}/{}/", TEST_DIRECTORY, GIT_DIR, REF_HEADS);

        if let Err(err) = fs::create_dir_all(&branch_path) {
            panic!("Falló al crear el directorio temporal: {}", err);
        }

        let branch_path_file = format!(
            "{}/{}/{}/{}",
            TEST_DIRECTORY, GIT_DIR, REF_HEADS, branch_name
        );
        fs::File::create(&branch_path_file)
            .expect("Falló al crear el archivo que contiene la branch");

        // Cuando ejecuta la función
        let result = git_branch_list(TEST_DIRECTORY);

        // Limpia el archivo de prueba
        if !Path::new(TEST_DIRECTORY).exists() {
            fs::remove_dir_all(TEST_DIRECTORY).expect("Falló al remover el directorio temporal");
        }

        // Entonces la función no lanza error.
        assert!(result.is_ok());
    }

    #[test]
    fn test_git_branch_create() {
        let branch_path = format!("{}/{}/{}/", TEST_DIRECTORY, GIT_DIR, REF_HEADS);
        if let Err(err) = fs::create_dir_all(&branch_path) {
            panic!("Falló al crear el directorio temporal: {}", err);
        }
        let _ = git_branch_delete(TEST_DIRECTORY, "test_new_branch");
        // Cuando ejecuto la función
        let result = git_branch_create(TEST_DIRECTORY, "test_new_branch");
        // Limpia el archivo de prueba
        if !Path::new(TEST_DIRECTORY).exists() {
            fs::remove_dir_all(TEST_DIRECTORY).expect("Falló al remover el directorio temporal");
        }

        // Entonces la función no lanza error.
        assert!(result.is_ok());
    }

    #[test]
    fn test_git_branch_delete() {
        let branch_path = format!("{}/{}/{}/", TEST_DIRECTORY, GIT_DIR, REF_HEADS);
        if let Err(err) = fs::create_dir_all(&branch_path) {
            panic!("alló al crear el directorio temporal: {}", err);
        }

        // Crea una rama ficticia
        let branch_name = "test_branch_delete";
        let branch_path = format!(
            "{}/{}/{}/{}",
            TEST_DIRECTORY, GIT_DIR, REF_HEADS, branch_name
        );
        fs::File::create(&branch_path).expect("Falló al crear el archivo que contiene la branch");

        // Cuando ejecuto la función
        let result = git_branch_delete(TEST_DIRECTORY, branch_name);

        // Entonces la función no lanza error.
        assert!(result.is_ok());

        // Entonces la rama ha sido eliminada.
        assert!(fs::metadata(&branch_path).is_err());

        // Limpia el archivo de prueba
        if !Path::new(TEST_DIRECTORY).exists() {
            fs::remove_dir_all(TEST_DIRECTORY).expect("Falló al remover el directorio temporal");
        }
    }

    #[test]
    fn test_get_current_branch() -> Result<(), GitError> {
        git_init(TEST_DIRECTORY)?;
        git_branch_create(TEST_DIRECTORY, "test_branch3")?;
        git_checkout_switch(TEST_DIRECTORY, "test_branch3")?;
        let result = get_current_branch(TEST_DIRECTORY);
        assert_eq!(result, Ok("test_branch3".to_string()));

        // Limpia el archivo de prueba
        if !Path::new(TEST_DIRECTORY).exists() {
            fs::remove_dir_all(TEST_DIRECTORY).expect("Falló al remover el directorio temporal");
        }
        Ok(())
    }
}
