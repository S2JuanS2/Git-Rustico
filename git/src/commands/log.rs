use crate::consts::*;
use crate::errors::GitError;
use crate::models::client::Client;
use std::path::Path;

const BRANCH_DIR: &str = "refs/heads/";


/// Esta función se encarga de llamar al comando log con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función log
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_log(args: Vec<&str>, client: Client) -> Result<(), GitError> {
    if args.len() > 1 {
        return Err(GitError::InvalidArgumentCountLogError);
    }
    let directory = client.get_directory_path();
    git_log(&directory)
}

/// muestra el log de los commits
/// ###Parametros:
/// 'directory': directorio del repositorio local 
pub fn git_log(directory: &str) -> Result<(), GitError> {
    let log = String::new();
    let commit = String::new();
    let author = String::new();
    let date = String::new();
    let message = String::new();
    let file = String::new();
    let line = String::new();
    let line_number = String::new();
    let line_count = String::new();
    let directory_git = format!("{}{}", directory, GIT_DIR);
    let _branch_dir = Path::new(&directory_git).join(BRANCH_DIR);
    println!("{}", log);
    println!("{}", commit);
    println!("{}", author);
    println!("{}", date);
    println!("{}", message);
    println!("{}", file);
    println!("{}", line);
    println!("{}", line_number);
    println!("{}", line_count);
    Ok(())

}