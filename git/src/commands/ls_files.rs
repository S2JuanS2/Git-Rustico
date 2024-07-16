use super::errors::CommandsError;
use crate::models::client::Client;

use super::status::{
    check_for_deleted_files, compare_hash_lists, get_hashes_index, get_hashes_working_directory,
    get_index_content, get_lines_in_index,
};

/// Esta función se encarga de llamar a al comando ls-files con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función ls-files
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_ls_files(args: Vec<&str>, client: Client) -> Result<String, CommandsError> {
    if args.len() > 1 {
        return Err(CommandsError::InvalidArgumentCountLsFilesError);
    }
    if args.len() == 1 && args[0] != "-c" && args[0] != "-d" && args[0] != "-m" && args[0] != "-o" {
        return Err(CommandsError::FlagLsFilesNotRecognizedError);
    }
    let directory = client.get_directory_path();
    if args.is_empty() {
        git_ls_files(directory, "")
    } else {
        git_ls_files(directory, args[0])
    }
}

/// Esta función se encarga de listar los archivos que se encuentran en el index.
/// ###Parametros:
/// 'directory': directorio del repositorio local.
/// 'flag': flag que se pasa por parametro.
pub fn git_ls_files(directory: &str, flag: &str) -> Result<String, CommandsError> {
    let directory_git = format!("{}/.git", directory);
    let mut formatted_result = String::new();
    let index_content = get_index_content(&directory_git)?;

    if flag.is_empty() || flag == "-c" {
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

/// Esta función se encarga de listar los archivos que se eliminaron del directorio local pero siguen
/// en el index.
/// ###Parametros:
/// 'directory': directorio del repositorio local.
/// 'index_content': contenido del index.
/// 'formatted_result': string que contiene el resultado formateado.
fn get_deleted_files(
    directory: &str,
    index_content: &str,
    formatted_result: &mut String,
) -> Result<(), CommandsError> {
    let working_directory_hash_list = get_hashes_working_directory(directory)?;
    let index_lines = get_lines_in_index(index_content.to_string());
    let index_hashes = get_hashes_index(index_lines)?;

    let deleted_files =
        check_for_deleted_files(&index_hashes, &working_directory_hash_list, directory);
    for file in deleted_files.iter() {
        formatted_result.push_str(&format!("{}\n", file));
    }

    Ok(())
}

/// Dependiendo si la flag es '-m' o '-o', esta función se encarga de listar los archivos que se
/// modificaron pero no se actualizaron los cambios en el index, o los archivos que no estan trackeados.
/// ###Parametros:
/// 'directory': directorio del repositorio local.
/// 'index_content': contenido del index.
/// 'formatted_result': string que contiene el resultado formateado.
/// 'flag': flag que se pasa por parametro.
fn get_modified_or_untracked_files(
    directory: &str,
    index_content: &str,
    formatted_result: &mut String,
    flag: &str,
) -> Result<(), CommandsError> {
    let working_directory_hash_list = get_hashes_working_directory(directory)?;
    let index_lines = get_lines_in_index(index_content.to_string());
    let index_hashes = get_hashes_index(index_lines)?;

    let status_data = compare_hash_lists(&working_directory_hash_list, &index_hashes, directory)?;

    let updated_files_list = status_data.updated_files_list();
    let untracked_files_list = status_data.untracked_files_list();
    let deleted_files_list = status_data.deleted_files_list();

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
    use super::*;
    use crate::commands::{add::git_add, init::git_init};
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
        file2
            .write_all(b"Hola Mundo file2")
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
        file2
            .write_all(b"Hola Mundo file2")
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
        file2
            .write_all(b"Hola Mundo file2")
            .expect("Error al escribir en el archivo");

        git_add(directory, "file1.rs").expect("Error al agregar el archivo");

        let result = git_ls_files(directory, "-c").expect("Error al ejecutar el comando");
        assert_eq!(result, "file1.rs\n");

        let result = git_ls_files(directory, "-o").expect("Error al ejecutar el comando");
        assert_eq!(result, "file2.rs\n");

        fs::remove_dir_all(directory).expect("Error al intentar remover el directorio");
    }
}