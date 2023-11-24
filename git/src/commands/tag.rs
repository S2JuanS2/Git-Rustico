use crate::consts::GIT_DIR;
use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::files::{open_file, read_file_string, create_file};
use crate::util::formats::hash_generate;

use super::branch::get_current_branch;

const BRANCH_DIR: &str = "refs/heads/";

use std::fs;

/// Esta función se encarga de llamar a al comando tag con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función tag
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_tag(args: Vec<&str>, client: Client) -> Result<String, GitError> {
    if !args.is_empty() {
        return Err(GitError::InvalidArgumentShowRefError);
    }
    let directory = client.get_directory_path();
    git_tag(directory)
}

/// Ejecuta las funciones para crear el objeto Tag.
/// ###Parametros:
/// 'directory': directorio del repositorio local.
pub fn git_tag(directory: &str) -> Result<String, GitError> {
    
    let git_dir = format!("{}/{}", directory, GIT_DIR);
    let current_branch = get_current_branch(directory)?;
    let branch_current_path = format!("{}/{}{}", git_dir, BRANCH_DIR, current_branch);

    let mut commit_hash = String::new();
    if fs::metadata(&branch_current_path).is_ok() {
        let file = open_file(&branch_current_path)?;
        commit_hash = read_file_string(file)?;
    }

    let tag_name = "v1.0.0";
    let tag_content = format!(
        "object {}\ntype commit\ntag {}\ntagger User <user@gmail.com> {} +0000\n\nRelease v1.0.0",
        tag_name,
        commit_hash,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
    );

    let tag_hash = hash_generate(&tag_content);

    let dir_tag = format!("{}/.git/refs/tags/{}", directory, tag_name);

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
