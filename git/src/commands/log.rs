use crate::errors::GitError;
use crate::models::client::Client;
use std::fs::File;
use std::io::{BufRead, BufReader};

use super::branch::get_current_branch;

/// Esta función se encarga de llamar al comando log con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función log
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_log(args: Vec<&str>, client: Client) -> Result<String, GitError> {
    if args.len() > 1 {
        return Err(GitError::InvalidArgumentCountLogError);
    }
    let directory = client.get_directory_path();
    git_log(directory)
}

/// Muestra el log de los commits
/// ###Parametros:
/// 'directory': directorio del repositorio local
pub fn git_log(directory: &str) -> Result<String, GitError> {
    let mut formatted_result = String::new();

    let logs_path = format!("{}/.git/logs/refs/heads", directory);

    let current_branch = get_current_branch(directory)?;

    let commit_file = format!("{}/{}", logs_path, current_branch);
    if let Ok(file) = File::open(commit_file) {
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader
            .lines()
            .map(|line| {
                line.map_err(|_| GitError::ReadFileError)
                    .unwrap_or_else(|_| String::new())
            })
            .collect();

            get_parts_commit(lines, &mut formatted_result);
    }

    Ok(formatted_result)
}

/// Obtiene las partes del commit.
/// ###Parametros:
/// 'lines': Vector de strings que contiene las lineas del archivo del commit
/// 'formatted_result': String que contiene el resultado de git log formateado
fn get_parts_commit(lines: Vec<String>, formatted_result: &mut String) {
    let mut count_line = 0;
    for line in lines{
        if count_line == 1{
            let parts: Vec<&str> = line.split_whitespace().collect();
            formatted_result.push_str(&format!("Commit: {}\n", parts[0]));
        }else if count_line == 3{
            let parts: Vec<&str> = line.split_whitespace().collect();
            formatted_result.push_str(&format!("Author: {} {}\n", parts[1], parts[2]));
        }else if count_line == 6{
            formatted_result.push('\n');
            formatted_result.push_str(&format!("{}\n",line));
        }
        count_line += 1;
        if count_line == 7{
            count_line = 1;
            formatted_result.push('\n');
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::add::git_add;
    use crate::commands::commit::{git_commit, Commit};
    use crate::commands::init::git_init;
    use crate::util::files::create_file;

    use super::*;
    use std::fs;

    #[test]
    fn test_git_log() {
        let directory = "./test_log";
        git_init(directory).expect("Falló al crear el repositorio");

        let file_path = format!("{}/test.txt", directory);
        create_file(&file_path, "test").expect("Falló al crear el archivo");

        git_add(directory, "test.txt").expect("Falló al agregar el primer archivo");

        let test_commit1 = Commit::new(
            "prueba".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );

        git_commit(directory, test_commit1).expect("Falló al hacer el primer commit");

        let other_file_path = format!("{}/test2.txt", directory);
        create_file(&other_file_path, "test2").expect("Falló al crear el archivo");

        git_add(directory, "test2.txt").expect("Falló al agregar el segundo archivo");

        let test_commit1 = Commit::new(
            "prueba2".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );

        git_commit(directory, test_commit1).expect("Falló al hacer el segundo commit");

        let result = git_log(directory);

        assert!(result.is_ok());

        fs::remove_dir_all(directory).expect("Falló al remover el directorio temporal");
    }
}
