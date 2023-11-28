use std::fs;
use std::io::BufRead;

use super::errors::CommandsError;
use crate::models::client::Client;
use crate::util::files::{open_file, read_file_string};

/// Esta función se encarga de llamar a al comando check-ignore con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función check-ignore
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_check_ignore(args: Vec<&str>, client: Client) -> Result<String, CommandsError> {
    if args.is_empty() {
        return Err(CommandsError::InvalidArgumentCountCheckIgnoreError);
    }
    let directory = client.get_directory_path();
    if args.len() == 1 {
        return git_check_ignore(directory, vec![args[0]]);
    }
    git_check_ignore(directory, args)
}

/// Verifica si los archivos o directorios pasados como parametro estan incluidos en .gitignore.
/// ###Parametros:
/// 'directory': directorio del repositorio local.
/// 'paths': Vector de strings que contiene los paths a verificar
pub fn git_check_ignore(directory: &str, paths: Vec<&str>) -> Result<String, CommandsError> {
    let mut ignored_files = Vec::<String>::new();
    let mut formatted_result = String::new();

    let gitignore_content = get_gitignore_content(directory)?;

    if paths.len() == 1 && paths[0] == "--stdin" {
        let stdin = std::io::stdin();
        let lines = stdin.lock().lines();
        lines.flatten().for_each(|line| {
            check_gitignore(&line, &mut ignored_files, &gitignore_content).unwrap();
        });
        for ignored_file in ignored_files {
            formatted_result.push_str(format!("{}\n", ignored_file).as_str());
        }
        return Ok(formatted_result);
    }

    for path in paths {
        check_gitignore(path, &mut ignored_files, &gitignore_content)?;
    }

    for ignored_file in ignored_files {
        formatted_result.push_str(format!("{}\n", ignored_file).as_str());
    }
    Ok(formatted_result)
}

/// Obtiene el contenido del archivo .gitignore.
/// ###Parametros:
/// 'directory': directorio del repositorio local.
pub fn get_gitignore_content(directory: &str) -> Result<String, CommandsError> {
    let gitignore_path = format!("{}/.gitignore", directory);
    if fs::metadata(&gitignore_path).is_err() {
        return Ok(String::new());
    }
    let gitignore = open_file(&gitignore_path)?;
    let gitignore_content = read_file_string(gitignore)?;
    Ok(gitignore_content)
}

/// Verifica si un path esta incluido en .gitignore.
/// ###Parametros:
/// 'path_to_check': path a verificar.
/// 'ignored_files': resultado del git check-ignore.
/// 'directory': directorio del repositorio local.
pub fn check_gitignore(
    path_to_check: &str,
    ignored_files: &mut Vec<String>,
    gitignore_content: &str,
) -> Result<(), CommandsError> {
    let gitignore_lines: Vec<&str> = gitignore_content.lines().collect();

    if gitignore_lines.contains(&path_to_check) {
        ignored_files.push(path_to_check.to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::util::files::create_file_replace;
    use std::fs;

    use super::*;

    #[test]
    fn test_git_check_ignore_paths() {
        let directory = "./test_check_ignore_paths";
        fs::create_dir_all(directory).expect("Error al crear el directorio");
        let path = format!("{}/.gitignore", directory);
        create_file_replace(&path, "target/\nCargo.lock\n").expect("Error al crear el archivo");

        let args = vec!["target/"];
        let result_con_un_path = git_check_ignore(directory, args);
        assert!(result_con_un_path.is_ok());

        let args = vec!["target/", "Cargo.lock", "Cargo.toml"];
        let result_con_varios_paths = git_check_ignore(directory, args);
        assert!(result_con_varios_paths.is_ok());

        fs::remove_dir_all(directory).expect("Error al eliminar el directorio");
    }
}
