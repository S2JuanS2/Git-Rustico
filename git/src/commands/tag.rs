use crate::consts::GIT_DIR;
use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::files::{open_file, read_file_string, create_file, delete_file};
use crate::util::formats::hash_generate;

use super::branch::get_current_branch;

const BRANCH_DIR: &str = "refs/heads/";

use std::fs;

//git tag -> muestra todas las tags
// git tag -a v1.0 -m "Version 1.0" -> Crea nueva tag anotada
//git show v1.0 -> muestra la información de la tag
//git tag -d v1.0 -> elimina la tag


/// Esta función se encarga de llamar a al comando tag con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función tag
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_tag(args: Vec<&str>, client: Client) -> Result<String, GitError> {
    let directory = client.get_directory_path();
    if args.is_empty() {
        git_tag_show()
    }else if args.len() == 4 && args[0] == "-a" {
        git_tag(directory, client.clone(), args[1], args[3])
    }else if args.len() == 2 && args[0] == "-d" {
        git_tag_delete(directory, args[1])
    }else{
        return Err(GitError::InvalidArgumentCountTagError)
    }
}

fn git_tag_delete(directory: &str, tag_name_delete: &str) -> Result<String, GitError> {

    let dir_tag = format!("{}/.git/refs/tags/{}", directory, tag_name_delete);

    delete_file(&dir_tag)?;

    Ok("Sucessfuly".to_string())
}

fn git_tag_show() -> Result<String, GitError> {


    Ok("Sucessfuly".to_string())
}

/// Ejecuta las funciones para crear el objeto Tag.
/// ###Parametros:
/// 'directory': directorio del repositorio local.
pub fn git_tag(directory: &str, client: Client, tag_name: &str, version_name: &str) -> Result<String, GitError> {
    
    let git_dir = format!("{}/{}", directory, GIT_DIR);
    let current_branch = get_current_branch(directory)?;
    let branch_current_path = format!("{}/{}{}", git_dir, BRANCH_DIR, current_branch);

    let mut commit_hash = String::new();
    if fs::metadata(&branch_current_path).is_ok() {
        let file = open_file(&branch_current_path)?;
        commit_hash = read_file_string(file)?;
    }

    let tag_content = format!(
        "object {}\ntype commit\ntag {}\n{} <{}> {} +0000\n\nRelease {}",
        client.get_name(),
        client.get_email(),
        tag_name,
        commit_hash,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
        version_name,
    );

    let tag_hash = hash_generate(&tag_content);

    let dir_tag = format!("{}/.git/refs/tags/{}", directory, version_name);

    create_file(&dir_tag, &tag_hash)?;

    println!("{}", tag_hash);

    Ok("Sucessfuly".to_string())
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag() {
        git_tag("Repository");
    }
}
*/
