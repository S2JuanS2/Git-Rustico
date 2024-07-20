use super::add::add_to_index;
use super::branch::get_branch;
use super::branch::get_current_branch;
use super::branch::git_branch_create;
use super::cat_file::git_cat_file;
use super::status::is_files_to_commit;
use crate::consts::*;
use crate::util::files::is_folder_empty;
use super::errors::CommandsError;

use crate::models::client::Client;
use crate::util::files::create_directory;
use crate::util::files::create_file_replace;
use crate::util::files::open_file;
use crate::util::files::read_file_string;
use crate::util::index::empty_index;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

/// Esta función se encarga de llamar al comando checkout con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función checkout
/// 'client': Cliente que contiene el directorio del repositorio local.
pub fn handle_checkout(args: Vec<&str>, client: Client) -> Result<String, CommandsError> {
    let directory = client.get_directory_path();
    if args.len() == 1 {
        Ok(git_checkout_switch(directory, args[0])?)
    } else if args.len() == 2 {
        if args[0] == "-b" {
            git_branch_create(directory, args[1])?;
            Ok(git_checkout_switch(directory, args[1])?)
        } else {
            return Err(CommandsError::FlagCheckoutNotRecognisedError);
        }
    } else {
        return Err(CommandsError::InvalidArgumentCountCheckoutError);
    }
}

/// Esta función se encarga de leer el tree hash de un commit
/// ###Parametros:
/// 'content_commit': Contenido de un commit
pub fn get_tree_hash(content_commit: &str) -> Option<&str> {
    if let Some(pos) = content_commit.find("tree ") {
        let start = pos + "tree ".len();

        if let Some(end) = content_commit[start..].find(char::is_whitespace) {
            return Some(&content_commit[start..start + end]);
        }
    }
    None
}

/// Esta función se encarga de cargar los archivos de un tree en el directorio
/// ###Parametros:
/// 'directory': directorio del repositorio local.
/// 'tree_hash': Valor hash de 40 caracteres (SHA-1) del tree a leer.
fn load_files(
    directory: &str,
    tree_hash: &str,
    mode: usize,
    dir_path: &str,
) -> Result<(), CommandsError> {
    let tree = git_cat_file(directory, tree_hash, "-p")?;
    for line in tree.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let file_mode;
        let path_file;
        if parts[0] == FILE || parts[0] == DIRECTORY {
            file_mode = parts[0];
            path_file = parts[1];
        }else{
            path_file = parts[0];
            file_mode = parts[1];
        }
        let hash = parts[2];
        let path_file_format = format!("{}/{}/{}", directory, dir_path, path_file);
        if file_mode == FILE {
            let content_file = git_cat_file(directory, hash, "-p")?;

            if mode == 0 {
                create_file_replace(&path_file_format, &content_file)?;
                let git_dir = format!("{}/{}", directory, GIT_DIR);
                let file_name = format!("{}/{}", dir_path, path_file);
                add_to_index(git_dir, &file_name[1..], hash.to_string())?;
            } else if mode == 1
                && fs::metadata(&path_file_format).is_ok()
                && fs::remove_file(&path_file_format).is_err()
            {
                return Err(CommandsError::RemoveFileError);
            }
        } else if file_mode == DIRECTORY {
            create_directory(Path::new(&path_file_format))?;
            let new_path = format!("{}/{}", dir_path, path_file);
            load_files(directory, hash, mode, &new_path)?;
            let new_path_dir = format!("{}/{}", directory, new_path);
            if mode == 1 && is_folder_empty(&new_path_dir)? && fs::remove_dir(new_path_dir).is_err() {
                return Err(CommandsError::RemoveFileError);
            }
        }
    }
    Ok(())
}

/// Esta función se encarga de leer el parent hash de un commit
/// ###Parametros:
/// 'commit': Contenido de un commit
pub fn extract_parent_hash(commit: &str) -> Option<&str> {
    for line in commit.lines() {
        if line.starts_with("parent") {
            let words: Vec<&str> = line.split_whitespace().collect();
            return words.get(1).copied();
        }
    }
    None
}

/// Esta función se encarga de leer los commits padres de un commit recursivamente
/// ###Parametros:
/// 'directory': directorio del repositorio local.
/// 'hash_commit': Valor hash de 40 caracteres (SHA-1) del commit a leer.
pub fn read_parent_commit(directory: &str, hash_commit: &str, mode: usize) -> Result<(), CommandsError> {
    let commit = git_cat_file(directory, hash_commit, "-p")?;

    if let Some(tree_hash) = get_tree_hash(&commit) {
        load_files(directory, tree_hash, mode, "")?;
    } else {
        return Err(CommandsError::GetHashError);
    };

    Ok(())
}

/// Esta función se encarga de leer el commit de un branch y sus padres.
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
/// 'branch_name': Nombre de la branch a cambiar.
fn load_files_tree(directory: &str, branch_name: &str, mode: usize) -> Result<(), CommandsError> {
    let branch = format!("{}/{}/{}/{}", directory, GIT_DIR, REF_HEADS, branch_name);

    let file = open_file(&branch)?;
    let hash_commit = read_file_string(file)?;

    read_parent_commit(directory, &hash_commit, mode)?;
    Ok(())
}

/// Cambia a otra branch existente
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
/// 'branch_name': Nombre de la branch a cambiar.
pub fn git_checkout_switch(directory: &str, branch_switch_name: &str) -> Result<String, CommandsError> {
    //Falta implementar que verifique si realizó commit ante la pérdida de datos. <- con el status..
    let current_branch_name = get_current_branch(directory)?;

    if current_branch_name == branch_switch_name {
        return Err(CommandsError::AlreadyOnThatBranch)
    }

    let branches = get_branch(directory)?;
    if !branches.contains(&branch_switch_name.to_string()) {
        return Err(CommandsError::BranchNotFoundError);
    }
    
    if is_files_to_commit(directory)? {
        return Ok("Please commit your changes\nAborting".to_string())
    }
    
    let directory_git = format!("{}/{}", directory, GIT_DIR);
    let head_file_path = Path::new(&directory_git).join(HEAD);

    let mut file = match OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(head_file_path)
    {
        Ok(file) => file,
        Err(_) => return Err(CommandsError::BranchDirectoryOpenError),
    };

    let content = format!("ref: refs/heads/{}\n", branch_switch_name);
    if file.write_all(content.as_bytes()).is_err() {
        return Err(CommandsError::BranchFileWriteError);
    }
    empty_index(directory)?;
    load_files_tree(directory, &current_branch_name, 1)?;
    load_files_tree(directory, branch_switch_name, 0)?;

    let response = format!("Switched to branch {}", branch_switch_name);
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        commands::{
            add::git_add,
            branch::git_branch_create,
            commit::{git_commit, Commit},
            init::git_init,
        },
        util::files::create_file,
    };
    use std::fs;

    #[test]
    fn test_git_checkout_switch_error() {
        let directory = "./test_git_checkout_switch_error";
        git_init(directory).expect("Falló al inicializar el repositorio");

        let current_branch_path = format!("{}/{}/{}/{}", directory, GIT_DIR, REF_HEADS, "master");
        create_file(current_branch_path.as_str(), "12345")
            .expect("Falló al crear el archivo que contiene la branch");

        // Cuando ejecuto la función sin agregar la branch "test_branch_switch1"
        let result = git_checkout_switch(directory, "test_branch_switch1");

        fs::remove_dir_all(directory).expect("Falló al remover el directorio temporal");

        // Entonces la función lanza error
        assert!(result.is_err());
    }

    #[test]
    fn test_git_checkout_switch_ok() {
        let directory = "./test_git_checkout_switch_ok";
        git_init(directory).expect("Falló al inicializar el repositorio");

        let file_path = format!("{}/{}", directory, "hola_mundo.txt");
        let mut file = fs::File::create(&file_path).expect("Falló al crear el archivo");
        file.write_all(b"hola mundo")
            .expect("Error al escribir en el archivo");

        let test_commit = Commit::new(
            "prueba".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );

        git_add(directory, "hola_mundo.txt").expect("Falló al agregar el archivo");
        git_commit(directory, test_commit).expect("Falló al hacer el commit");

        git_branch_create(directory, "test_branch_switch2")
            .expect("Falló en la creación de la branch");

        let result = git_checkout_switch(directory, "test_branch_switch2");

        let head_file = format!("{}/{}/{}", directory, GIT_DIR, HEAD);
        let head_file_path = open_file(&head_file).expect("Falló al abrir el archivo");
        let head_actualizado = read_file_string(head_file_path).expect("Falló al leer el archivo");

        fs::remove_dir_all(directory).expect("Falló al remover el directorio temporal");

        assert!(result.is_ok());
        assert_eq!(head_actualizado, "ref: /refs/heads/test_branch_switch2\n")
    }
}
