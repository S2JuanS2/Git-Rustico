use super::branch::get_branch;
use super::branch::git_branch_create;
use super::cat_file::git_cat_file;
use crate::consts::*;
use crate::errors::GitError;

use crate::models::client::Client;
use crate::util::files::create_directory;
use crate::util::files::create_file_replace;
use crate::util::files::open_file;
use crate::util::files::read_file_string;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

/// Esta función se encarga de llamar al comando checkout con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función checkout
/// 'client': Cliente que contiene el directorio del repositorio local.
pub fn handle_checkout(args: Vec<&str>, client: Client) -> Result<String, GitError> {
    let directory = client.get_directory_path();
    if args.len() == 1 {
        git_checkout_switch(directory, args[0])?;
    } else if args.len() == 2 {
        if args[0] == "-b" {
            git_branch_create(directory, args[1])?;
            git_checkout_switch(directory, args[1])?;
        } else {
            return Err(GitError::FlagCheckoutNotRecognisedError);
        }
    } else {
        return Err(GitError::InvalidArgumentCountCheckoutError);
    }
    Ok("Rama cambiada con éxito".to_string())
}

fn get_tree_hash(contenido: &str) -> Option<&str> {

    if let Some(pos) = contenido.find("tree ") {
        let start = pos + "tree ".len();

        if let Some(end) = contenido[start..].find(char::is_whitespace) {
            return Some(&contenido[start..start + end]);
        }
    }

    // Devolver None si no se encuentra ninguna coincidencia
    None
}

fn load_files_tree(directory: &str, branch_name: &str) -> Result<(),GitError>{

    let branch = format!("{}/{}/{}/{}", directory, GIT_DIR, REF_HEADS, branch_name);

    let file = open_file(&branch)?;
    let hash_commit = read_file_string(file)?;

    let commit = git_cat_file(directory, &hash_commit, "-p")?;

    if let Some(tree_hash) = get_tree_hash(&commit){

        let tree = git_cat_file(directory, &tree_hash, "-p")?;

        for line in tree.lines() {

            let parts: Vec<&str> = line.split_whitespace().collect();

            let path_file = parts[1];
            let hash_blob = parts[2];

            let path_file_format = format!("{}/{}", directory, path_file);
            let content_file = git_cat_file(directory, hash_blob, "-p")?;

            let path = Path::new(&path_file_format);

            if let Some(parent) = path.parent(){
                create_directory(parent)?;

            }
            create_file_replace(&path_file_format, &content_file)?;
        }
    };
    Ok(())
}

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

    let directory_git = format!("{}/{}", directory, GIT_DIR);
    let head_file_path = Path::new(&directory_git).join(HEAD);

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

    load_files_tree(directory, branch_name)?;

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
        let branch_path = format!("{}/{}/{}", TEST_DIRECTORY, GIT_DIR, BRANCH_DIR);
        if let Err(err) = fs::create_dir_all(&branch_path) {
            panic!("Falló al crear el directorio: {}", err);
        }
        // Cuando ejecuto la función
        let result = git_checkout_switch(TEST_DIRECTORY, "test_branch_switch1");

        // Limpia el archivo de prueba
        if !Path::new(TEST_DIRECTORY).exists() {
            fs::remove_dir_all(TEST_DIRECTORY).expect("Falló al remover el directorio temporal");
        };

        // Entonces la función lanza error
        assert!(result.is_err());
    }

    #[test]
    fn test_git_checkout_switch_ok() {
        let branch_path = format!("{}/{}/{}", TEST_DIRECTORY, GIT_DIR, BRANCH_DIR);
        if let Err(err) = fs::create_dir_all(&branch_path) {
            panic!("Falló al crear el directorio: {}", err);
        }
        let _ = git_branch_delete(TEST_DIRECTORY, "test_branch_switch2");
        git_branch_create(TEST_DIRECTORY, "test_branch_switch2")
            .expect("Falló en la creación de la branch");
        // Cuando ejecuto la función
        let result = git_checkout_switch(TEST_DIRECTORY, "test_branch_switch2");

        // Limpia el archivo de prueba
        if !Path::new(TEST_DIRECTORY).exists() {
            fs::remove_dir_all(TEST_DIRECTORY).expect("Falló al remover el directorio temporal");
        }

        // Entonces la función no lanza error.
        assert!(result.is_ok());
    }
}
