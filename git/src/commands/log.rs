use crate::consts::*;
use crate::errors::GitError;
use crate::models::client::Client;
use std::fs::File;
use std::io::{BufRead, BufReader};

use super::branch::get_current_branch;


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
    let logs_path = format!("{}/.git/logs/refs/heads", directory);

    let current_branch = get_current_branch(directory)?;

    let commit_file = format!("{}/{}", logs_path, current_branch);
    if let Ok(file) = File::open(&commit_file) {
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader
            .lines()
            .map(|line| line.map_err(|_| GitError::ReadFileError).unwrap_or_else(|_| String::new()))
            .collect();

        let lines_per_commit = 5;
        let total_lines = lines.len();
        let mut end_line = total_lines;

        while end_line > 0 {
            let start_line = end_line - lines_per_commit;

            println!("Commit:\n");

            for line_number in start_line..end_line {
                if line_number < total_lines {
                    println!("{}", &lines[line_number]);
                }
            }

            println!("----------------------\n");

            end_line = start_line;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_git_log() {
        let directory = "./testdir";
        if let Err(err) = fs::create_dir_all(&directory) {
            panic!("Falló al crear el directorio temporal: {}", err);
        }
        //crear head file
        let mut file = File::create(format!("{}/.git/HEAD", directory)).unwrap();
        writeln!(file, "ref: refs/heads/master").unwrap();
        let log_path = format!("{}/.git/logs/refs/heads/", directory);
        if let Err(err) = fs::create_dir_all(&log_path) {
            panic!("Falló al crear el directorio temporal: {}", err);
        }
        let mut file = File::create(format!("{}master", directory)).unwrap();
        writeln!(file, "Commit 2: Line 1").unwrap();
        writeln!(file, "Commit 2: Line 2").unwrap();
        writeln!(file, "Commit 2: Line 3").unwrap();
        writeln!(file, "Commit 2: Line 4").unwrap();
        writeln!(file, "Commit 2: Line 5").unwrap();
        writeln!(file, "Commit 1: Line 1").unwrap();
        writeln!(file, "Commit 1: Line 2").unwrap();
        writeln!(file, "Commit 1: Line 3").unwrap();
        writeln!(file, "Commit 1: Line 4").unwrap();
        writeln!(file, "Commit 1: Line 5").unwrap();
        let result = git_log(directory);
        assert!(result.is_ok());

        //eliminar head file
        if let Err(err) = fs::remove_file(format!("{}/.git/HEAD", directory)) {
            panic!("Falló al eliminar el archivo temporal: {}", err);
        }        

        if !Path::new(&log_path).exists() {
            fs::remove_dir_all(log_path).expect("Falló al remover el directorio temporal");
        }

        if !Path::new(directory).exists() {
            fs::remove_dir_all(directory).expect("Falló al remover el directorio temporal");
        }
    }
}