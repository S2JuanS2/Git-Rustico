use crate::errors::GitError;
use crate::models::client::Client;
use std::fs::File;
use std::io::{BufRead, BufReader};


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
    let logs_path = format!("{}logs/refs/heads", directory);

    let entries = match std::fs::read_dir(logs_path.clone()){
        Ok(entries) => entries,
        Err(_) => return Err(GitError::ReadDirError),
    };

    for entry in entries.flatten() {
        if let Some(name) = entry.file_name().to_str() {
            let commit_file = format!("{}/{}", logs_path, name);
            if let Ok(file) = File::open(&commit_file){
                let reader = BufReader::new(file);                    
                for line in reader.lines().flatten() {
                        println!("{}", line);
                }
                println!("----------------------");
            }
        }
    }
    Ok(())

}