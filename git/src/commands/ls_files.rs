use crate::errors::GitError;
use crate::models::client::Client;

use super::status::{get_index_content, get_hashes_working_directory, get_lines_in_index, get_hashes_index, check_for_deleted_files, compare_hash_lists};

/// Esta función se encarga de llamar a al comando ls-files con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función ls-files
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_ls_files(args: Vec<&str>, client: Client) -> Result<String, GitError> {
    if args.len() > 1 {
        return Err(GitError::InvalidArgumentCountLsFilesError);
    }
    if args.len() == 1 && args[1] != "-c" && args[1] != "-d" && args[1] != "-m" && args[1] != "-o" {
        return Err(GitError::FlagLsFilesNotRecognizedError);
    }
    let directory = client.get_directory_path();
    if args.len() == 0 {
        return git_ls_files(directory, "");
    }
    else {
        return git_ls_files(directory, args[1]);
    }
}

pub fn git_ls_files(directory: &str, flag: &str) -> Result<String, GitError> {

    let directory_git = format!("{}/.git", directory);
    let mut formatted_result = String::new();
    let index_content = get_index_content(&directory_git)?;
    
    if flag == "" || flag == "-c" {
        let lines_index: Vec<String> = index_content.lines().map(String::from).collect();
        for line in lines_index {
            let parts: Vec<&str> = line.split_whitespace().collect();
            formatted_result.push_str(&format!("{}\n", parts[0]));
        }
    }
    if flag == "-d" {
        get_deleted_files(directory, &index_content, &mut formatted_result)?;
    }
    if flag == "-m" || flag == "-o" {
        get_modified_or_untracked_files(directory, &index_content, &mut formatted_result, flag)?;
    }

    Ok(formatted_result)
}

fn get_deleted_files(directory: &str, index_content: &str, formatted_result: &mut String) -> Result<(), GitError> {
    let working_directory_hash_list = get_hashes_working_directory(directory)?;
    let index_lines = get_lines_in_index(index_content.to_string());
    let index_hashes = get_hashes_index(index_lines)?;
    
    let deleted_files = check_for_deleted_files(&index_hashes, &working_directory_hash_list, directory);
    for file in deleted_files.iter() {
        formatted_result.push_str(&format!("{}\n", file));
    }

    Ok(())
}

fn get_modified_or_untracked_files(directory: &str, index_content: &str, formatted_result: &mut String, flag: &str) -> Result<(), GitError> {
    let working_directory_hash_list = get_hashes_working_directory(directory)?;
    let index_lines = get_lines_in_index(index_content.to_string());
    let index_hashes = get_hashes_index(index_lines)?;

    let (updated_files_list, untracked_files_list, 
        _staged_files_list, deleted_files_list) =
        compare_hash_lists(&working_directory_hash_list, &index_hashes, directory);

    if flag == "-m" {
        for file in updated_files_list.iter() {
            let file_path = &file.0[directory.len() + 1..];
            formatted_result.push_str(&format!("{}\n", file_path));
        }
        for file in deleted_files_list.iter() {
            formatted_result.push_str(&format!("{}\n", file));
        }
    }

    if flag == "-o" {
        for file in untracked_files_list.iter() {
            let file_path = &file.0[directory.len() + 1..];
            formatted_result.push_str(&format!("{}\n", file_path));
        }
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use crate::commands::{init::git_init, add::git_add};
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_git_ls_files() {
        let directory = "./test_ls_files";
        git_init(directory).expect("Error al crear el repositorio");

        let file_path = format!("{}/{}", directory, "file1.rs");
        let mut file = fs::File::create(&file_path).expect("Falló al crear el archivo");
        file.write_all(b"Hola Mundo file1")
            .expect("Error al escribir en el archivo");

        let file_path2 = format!("{}/{}", directory, "file2.rs");
        let mut file2 = fs::File::create(&file_path2).expect("Falló al crear el archivo");
        file2.write_all(b"Hola Mundo file2")
            .expect("Error al escribir en el archivo");

        git_add(directory, "file1.rs").expect("Error al agregar el archivo");
        git_add(directory, "file2.rs").expect("Error al agregar el archivo");

        let result = git_ls_files(directory, "").expect("Error al ejecutar el comando");
        assert_eq!(result, "file1.rs\nfile2.rs\n");

        let result = git_ls_files(directory, "-c").expect("Error al ejecutar el comando");
        assert_eq!(result, "file1.rs\nfile2.rs\n");
        
        fs::remove_dir_all(directory).expect("Error al intentar remover el directorio");
    }

    #[test]
    fn test_git_ls_files_modified() {
        let directory = "./test_ls_files_modified";
        git_init(directory).expect("Error al crear el repositorio");

        let file_path = format!("{}/{}", directory, "file1.rs");
        let mut file = fs::File::create(&file_path).expect("Falló al crear el archivo");
        file.write_all(b"Hola Mundo file1")
            .expect("Error al escribir en el archivo");

        let file_path2 = format!("{}/{}", directory, "file2.rs");
        let mut file2 = fs::File::create(&file_path2).expect("Falló al crear el archivo");
        file2.write_all(b"Hola Mundo file2")
            .expect("Error al escribir en el archivo");

        git_add(directory, "file1.rs").expect("Error al agregar el archivo");
        git_add(directory, "file2.rs").expect("Error al agregar el archivo");

        let result = git_ls_files(directory, "-c").expect("Error al ejecutar el comando");
        assert_eq!(result, "file1.rs\nfile2.rs\n");

        let result = git_ls_files(directory, "-m").expect("Error al ejecutar el comando");
        assert_eq!(result, "");

        let mut file = fs::File::create(&file_path).expect("Falló al crear el archivo");
        file.write_all(b"Hola Mundo file1 modificado")
            .expect("Error al escribir en el archivo");

        let result = git_ls_files(directory, "-m").expect("Error al ejecutar el comando");
        assert_eq!(result, "file1.rs\n");

        fs::remove_dir_all(directory).expect("Error al intentar remover el directorio");
    }

    #[test]
    fn test_git_ls_files_untracked() {
        let directory = "./test_ls_files_untracked";
        git_init(directory).expect("Error al crear el repositorio");

        let file_path = format!("{}/{}", directory, "file1.rs");
        let mut file = fs::File::create(&file_path).expect("Falló al crear el archivo");
        file.write_all(b"Hola Mundo file1")
            .expect("Error al escribir en el archivo");

        let file_path2 = format!("{}/{}", directory, "file2.rs");
        let mut file2 = fs::File::create(&file_path2).expect("Falló al crear el archivo");
        file2.write_all(b"Hola Mundo file2")
            .expect("Error al escribir en el archivo");

        git_add(directory, "file1.rs").expect("Error al agregar el archivo");

        let result = git_ls_files(directory, "-c").expect("Error al ejecutar el comando");
        assert_eq!(result, "file1.rs\n");

        let result = git_ls_files(directory, "-o").expect("Error al ejecutar el comando");
        assert_eq!(result, "file2.rs\n");

        fs::remove_dir_all(directory).expect("Error al intentar remover el directorio");
    }
}