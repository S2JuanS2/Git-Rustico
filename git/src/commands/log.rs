use super::cat_file::git_cat_file;
use super::checkout::extract_parent_hash;
use super::commit::builder_commit_log;
use super::errors::CommandsError;
use crate::consts::{PARENT_INITIAL, GIT_DIR};
use crate::models::client::Client;
use crate::util::files::{open_file, read_file_string};
use std::fs::File;
use std::io::{BufRead, BufReader};

use super::branch::get_current_branch;

/// Esta función se encarga de llamar al comando log con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función log
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_log(args: Vec<&str>, client: Client) -> Result<String, CommandsError> {
    if args.len() > 1 {
        return Err(CommandsError::InvalidArgumentCountLogError);
    }
    let directory = client.get_directory_path();
    git_log(directory)
}

/// Muestra el log de los commits
/// ###Parametros:
/// 'directory': directorio del repositorio local
pub fn git_log(directory: &str) -> Result<String, CommandsError> {
    let mut formatted_result = String::new();

    let logs_path = format!("{}/.git/logs/refs/heads", directory);

    let current_branch = get_current_branch(directory)?;

    let commit_file = format!("{}/{}", logs_path, current_branch);
    if let Ok(file) = File::open(commit_file) {
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader
            .lines()
            .map(|line| {
                line.map_err(|_| CommandsError::ReadFileError)
                    .unwrap_or_else(|_| String::new())
            })
            .collect();

        formatted_result = get_parts_commit(lines)?;
    }

    Ok(formatted_result)
}

/// Obtiene las partes del commit.
/// ###Parametros:
/// 'lines': Vector de strings que contiene las lineas del archivo del commit
/// 'formatted_result': String que contiene el resultado de git log formateado
pub fn get_parts_commit(lines: Vec<String>) -> Result<String, CommandsError> {
    let mut formatted_result = String::new();
       
    let mut count_line = 0;
    for line in lines {
        if count_line == 1 {
            let parts: Vec<&str> = line.split_whitespace().collect();
            formatted_result.push_str(&format!("Commit: {}\n", parts[0]));
        } else if count_line == 3 {
            let parts: Vec<&str> = line.split_whitespace().collect();
            formatted_result.push_str(&format!("Author: {} {}\n", parts[1], parts[2]));
            let timestamp = match parts[3].parse::<i64>(){
                Ok(t) => t,
                Err(_) => return Err(CommandsError::TimeStamp),
            };
            let date_time = chrono::DateTime::from_timestamp(timestamp, 0).unwrap();
            formatted_result.push_str(&format!("Date: {}\n", date_time));
        } else if count_line == 6 {
            formatted_result.push('\n');
            formatted_result.push_str(&format!("\t{}\n", line));
        }
        count_line += 1;
        if count_line == 7  {
            count_line = 1;
            formatted_result.push('\n');
        }
    }
    Ok(formatted_result)
}

/// Inserta una linea en una cadena recibida por parámetro
///
/// # Argumentos
///
/// - `original_string`: Cadena original a modificar
/// - `line_number_1`: Numero de linea a modificar
/// - `new_line`: Linea a agrear
///
pub fn insert_line_between_lines(
    original_string: &str,
    line_number_1: usize,
    new_line: &str,
) -> String {
    let mut result = String::new();

    let lines = original_string.lines();

    for (index, line) in lines.enumerate() {
        result.push_str(line);
        result.push('\n');
        if index + 1 == line_number_1 {
            let parent_format = format!("parent {}", new_line);
            result.push_str(&parent_format);
            result.push('\n');
        }
    }

    result
}

/// Recorre los parents de los commits y registra en el log de la branch
///
/// # Argumentos
///
/// - `directory`: Dirección del repositorio
/// - `commit`: Contenido del objeto commit
/// - `branch_name`: Nombre de la branch a modificar el log
///
/// # Returns
///
/// Un `Result` con un retorno `CommandsError` en caso de error.
///
pub fn save_parent_log(directory: &str, commit: &str, branch_name: &str, path_log: &str) -> Result<(), CommandsError> {

    if let Some(parent_hash) = extract_parent_hash(commit){
        if parent_hash != PARENT_INITIAL {
            let mut parent_commit = git_cat_file(directory, parent_hash, "-p")?;
            if parent_commit.lines().count() == 5{
                parent_commit = insert_line_between_lines(&parent_commit, 1, PARENT_INITIAL);
            }
            builder_commit_log(directory, &parent_commit, parent_hash, branch_name, path_log)?;
            save_parent_log(directory, &parent_commit, branch_name, path_log)?;

        }
    };

    Ok(())
}

/// Guarda los logs de los commits recibidos del servidor
///
/// # Argumentos
///
/// - `directory`: Dirección del repositorio
/// - `branch_name`: Nombre de la branch a modificar el log
///
/// # Returns
///
/// Un `Result` con un retorno `CommandsError` en caso de error.
///
pub fn save_log(directory: &str, branch_name: &str, path_log: &str, path_branch: &str) -> Result<(), CommandsError>{

    let dir_branch = format!("{}/{}/{}/{}", directory, GIT_DIR, path_branch, branch_name);
    let file = open_file(&dir_branch)?;
    let hash_commit = read_file_string(file)?;
    let mut commit_content = git_cat_file(directory, &hash_commit, "-p")?;
    if commit_content.lines().count() == 5{
        commit_content = insert_line_between_lines(&commit_content, 1, PARENT_INITIAL);
    }
    builder_commit_log(directory, &commit_content, &hash_commit, branch_name, path_log)?;
    save_parent_log(directory, &commit_content, branch_name, path_log)?;

    Ok(())
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
