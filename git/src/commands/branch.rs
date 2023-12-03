use crate::consts::*;
use super::errors::CommandsError;
use crate::models::client::Client;
use crate::util::files::{create_file, open_file, read_file, read_file_string};
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Esta función se encarga de llamar al comando branch con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de Strings que contiene los argumentos que se le pasaran al comando branch
/// 'client': Cliente que contiene el directorio del repositorio local
pub fn handle_branch(args: Vec<&str>, client: Client) -> Result<String, CommandsError> {
    let directory = client.get_directory_path();
    if args.is_empty() {
        git_branch_list(directory)
    } else if args.len() == 1 {
        git_branch_create(directory, args[0])
    } else if (args.len() == 2 && args[0] == "-d") || (args.len() == 2 && args[0] == "-D") {
        git_branch_delete(directory, args[1])
    } else {
        return Err(CommandsError::InvalidArgumentCountBranchError);
    }
}



/// Devuelve el hash de la branch recibida por parametro.
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
/// 'branch': nombre de la branch a obtener el hash.
pub fn get_branch_current_hash(directory: &str, branch: String) -> Result<String, CommandsError>{

    let dir_branch = format!("{}/{}/{}/remotes/origin/{}", directory, GIT_DIR, REFS, branch);
    let file = open_file(&dir_branch)?;
    let hash = read_file_string(file)?;

    Ok(hash)
}

/// Devuelve el nombre de la branch actual.
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
pub fn get_current_branch(directory: &str) -> Result<String, CommandsError> {
    let head_path = format!("{}/{}/HEAD", directory, GIT_DIR);
    let head_file = match File::open(head_path) {
        Ok(file) => file,
        Err(_) => return Err(CommandsError::BranchDirectoryOpenError),
    };

    let reader = BufReader::new(head_file);
    let mut branch = String::new();
    for line in reader.lines() {
        let line = match line {
            Ok(line) => line,
            Err(_) => return Err(CommandsError::BranchFileReadError),
        };
        let line_split: Vec<&str> = line.split('/').collect();
        branch = line_split[line_split.len() - 1].to_string();
    }

    Ok(branch)
}

/// Muestra en una etiqueta las branches existentes.
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
pub fn git_branch_list_display(directory: &str) -> Result<String, CommandsError> {
    let branches = get_branch(directory)?;
    let mut formatted_branches = String::new();
    for branch in branches {
        let dir_branch = format!("{}/{}/{}/heads/{}", directory, GIT_DIR, REFS, branch);
        let file = open_file(&dir_branch)?;
        let hash = read_file_string(file)?;
        formatted_branches.push_str(&format!(" - {} [{}]\n", branch, &hash[..7]))
    }
    Ok(formatted_branches)
}

/// Muestra por pantalla las branches existentes.
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
pub fn git_branch_list(directory: &str) -> Result<String, CommandsError> {
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

/// Copia el log de la branch actual a la nueva branch.
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
/// 'current_branch': Nombre de la branch actual.
/// 'branch_name': Nombre de la branch a crear.
pub fn copy_log(directory: &str, current_branch: &str, branch_name: &str) -> Result<(), CommandsError> {
    let current_branch_log_path = format!(
        "{}/{}/logs/refs/heads/{}",
        directory, GIT_DIR, current_branch
    );
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
pub fn git_branch_create(directory: &str, branch_name: &str) -> Result<String, CommandsError> {
    let branches = get_branch(directory)?;
    if branches.contains(&branch_name.to_string()) {
        return Err(CommandsError::BranchAlreadyExistsError);
    }
    let current_branch = get_current_branch(directory)?;
    let branch_current_path = format!("{}/{}/{}/{}", directory, GIT_DIR, REF_HEADS, current_branch);
    if fs::metadata(&branch_current_path).is_err() {
        return Err(CommandsError::BranchNotFoundError);
    }
    let file_current_branch = open_file(&branch_current_path)?;
    let hash_current_branch = read_file(file_current_branch)?;

    let commit_current_branch = match String::from_utf8(hash_current_branch) {
        Ok(commit_current_branch) => commit_current_branch,
        Err(_) => return Err(CommandsError::GenericError),
    };
    // Crear un nuevo archivo en .git/refs/heads/ con el nombre de la rama y el contenido es el hash del commit actual.
    let branch_path = format!("{}/{}/{}/{}", directory, GIT_DIR, REF_HEADS, branch_name);

    create_file(branch_path.as_str(), commit_current_branch.as_str())?;

    copy_log(directory, &current_branch, branch_name)?;

    let response = format!("Branch {} created", branch_name);

    Ok(response)
}

// Devuelve un vector con los nombres de las branchs
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
/// 'name_remote': Nombre del repositorio remoto.
pub fn get_branch_remote(directory: &str, name_remote: &str) -> Result<Vec<String>, CommandsError> {
    // "directory/.git/refs/remote/name_remote"
    let directory_git = format!("{}/{}", directory, GIT_DIR);
    let branch_dir = Path::new(&directory_git).join(format!("refs/remotes/{}", name_remote));

    let entries = match fs::read_dir(branch_dir) {
        Ok(entries) => entries,
        Err(_) => return Err(CommandsError::BranchDirectoryOpenError),
    };

    let mut branches: Vec<String> = Vec::new();

    for entry in entries {
        match entry {
            Ok(entry) => {
                let branch = match entry.file_name().into_string() {
                    Ok(branch) => branch,
                    Err(_) => return Err(CommandsError::ReadBranchesError),
                };
                branches.push(branch);
            }
            Err(_) => return Err(CommandsError::ReadBranchesError),
        }
    }
    Ok(branches)
}

// Devuelve un vector con los nombres de las branchs
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
pub fn get_branch(directory: &str) -> Result<Vec<String>, CommandsError> {
    // "directory/.git/refs/heads"
    let directory_git = format!("{}/{}", directory, GIT_DIR);
    let branch_dir = Path::new(&directory_git).join(REF_HEADS);

    let entries = match fs::read_dir(branch_dir) {
        Ok(entries) => entries,
        Err(_) => return Err(CommandsError::BranchDirectoryOpenError),
    };

    let mut branches: Vec<String> = Vec::new();

    for entry in entries {
        match entry {
            Ok(entry) => {
                let branch = match entry.file_name().into_string() {
                    Ok(branch) => branch,
                    Err(_) => return Err(CommandsError::ReadBranchesError),
                };
                branches.push(branch);
            }
            Err(_) => return Err(CommandsError::ReadBranchesError),
        }
    }

    Ok(branches)
}

/// Elimina una branch existente
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
/// 'branch_name': Nombre de la branch a eliminar.
pub fn git_branch_delete(directory: &str, branch_name: &str) -> Result<String, CommandsError> {
    if get_current_branch(directory) == Ok(branch_name.to_string()) {
        return Err(CommandsError::DeleteBranchError);
    }

    let branches = get_branch(directory)?;
    if !branches.contains(&branch_name.to_string()) {
        return Err(CommandsError::BranchNotFoundError);
    }

    // Crear un nuevo archivo en .git/refs/heads/ con el nombre de la rama y el contenido es el hash del commit actual.
    let branch_path = format!("{}/{}/{}/{}", directory, GIT_DIR, REF_HEADS, branch_name);

    if fs::remove_file(branch_path).is_err() {
        return Err(CommandsError::DeleteBranchError);
    }

    let response = format!("Branch {} deleted", branch_name);
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::init::git_init;
    use crate::util::files::create_file_replace;
    use std::fs;

    #[test]
    fn test_git_branch_list() {
        let directory = "./test_git_branch";
        git_init(directory).expect("Falló al inicializar el repositorio");

        let current_branch_path = format!("{}/{}/{}/{}", directory, GIT_DIR, REF_HEADS, "master");
        create_file(current_branch_path.as_str(), "12345")
            .expect("Falló al crear el archivo que contiene la branch");
        let branch_name = "test_branch";
        let branch_path = format!("{}/{}/{}/{}", directory, GIT_DIR, REF_HEADS, branch_name);
        create_file(branch_path.as_str(), "54321")
            .expect("Falló al crear el archivo que contiene la branch");

        let result = git_branch_list(directory);
        let test_branch = "- test_branch\n";
        let master_branch = "*- master\n";

        fs::remove_dir_all(directory).expect("Falló al remover el directorio temporal");

        assert!(result.is_ok());
        assert!(result.as_ref().unwrap().contains(test_branch));
        assert!(result.unwrap().contains(master_branch));
    }

    #[test]
    fn test_git_branch_create() {
        let directory = "./test_git_branch_create";
        git_init(directory).expect("Falló al inicializar el repositorio");

        let current_branch_path = format!("{}/{}/{}/{}", directory, GIT_DIR, REF_HEADS, "master");
        create_file(current_branch_path.as_str(), "12345")
            .expect("Falló al crear el archivo que contiene la branch");

        let logs_dir = format!("{}/{}/logs/refs/heads", directory, GIT_DIR);
        fs::create_dir_all(logs_dir).expect("Falló al crear el directorio de logs");

        let current_branch_log_path =
            format!("{}/{}/logs/refs/heads/{}", directory, GIT_DIR, "master");
        create_file(current_branch_log_path.as_str(), "12345")
            .expect("Falló al crear el archivo que contiene la branch");

        let result = git_branch_create(directory, "test_new_branch");
        let result_branch = format!("Branch {} created", "test_new_branch");

        fs::remove_dir_all(directory).expect("Falló al remover el directorio temporal");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), result_branch);
    }

    #[test]
    fn test_git_branch_delete() {
        let directory = "./test_git_branch_delete";
        let branch_path = format!("{}/{}/{}/", directory, GIT_DIR, REF_HEADS);
        if let Err(err) = fs::create_dir_all(&branch_path) {
            panic!("Falló al crear el directorio temporal: {}", err);
        }

        // Crea una rama ficticia
        let branch_name = "test_branch_delete";
        let branch_path = format!("{}/{}/{}/{}", directory, GIT_DIR, REF_HEADS, branch_name);
        fs::File::create(&branch_path).expect("Falló al crear el archivo que contiene la branch");

        let result = git_branch_delete(directory, branch_name);

        assert!(result.is_ok());

        assert!(fs::metadata(&branch_path).is_err());

        fs::remove_dir_all(directory).expect("Falló al remover el directorio temporal");
    }

    #[test]
    fn test_get_current_branch() {
        let directory = "./test_get_current_branch";
        git_init(directory).expect("Falló al inicializar el repositorio");

        let current_branch_path = format!("{}/{}/{}/{}", directory, GIT_DIR, REF_HEADS, "master");
        create_file(current_branch_path.as_str(), "12345")
            .expect("Falló al crear el archivo que contiene la branch");

        let logs_dir = format!("{}/{}/logs/refs/heads", directory, GIT_DIR);
        fs::create_dir_all(logs_dir).expect("Falló al crear el directorio de logs");

        let current_branch_log_path =
            format!("{}/{}/logs/refs/heads/{}", directory, GIT_DIR, "master");
        create_file(current_branch_log_path.as_str(), "12345")
            .expect("Falló al crear el archivo que contiene la branch");

        git_branch_create(directory, "test_branch3").expect("Falló al crear la rama");
        let head_file = format!("{}/{}/HEAD", directory, GIT_DIR);
        create_file_replace(head_file.as_str(), "ref: /refs/heads/test_branch3")
            .expect("Falló al actualizar el archivo HEAD");
        let result = get_current_branch(directory);

        fs::remove_dir_all(directory).expect("Falló al remover el directorio temporal");

        assert!(result.is_ok());
        assert_eq!(result, Ok("test_branch3".to_string()));
    }
}
