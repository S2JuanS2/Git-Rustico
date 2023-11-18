use crate::errors::GitError;
use crate::models::client::Client;

use super::status::{get_index_content, get_hashes_working_directory, get_lines_in_index, get_hashes_index, check_for_deleted_files};

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

    let mut formatted_result = String::new();
    let index_content = get_index_content(directory)?;
    
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
    if flag == "-m" {
        get_modified_files(directory, &index_content, &mut formatted_result);
    }
    if flag == "-o" {
        get_other_files(directory, &index_content, &mut formatted_result);
    }

    Ok(formatted_result)
}

fn get_deleted_files(directory: &str, index_content: &str, formatted_result: &mut String) -> Result<Vec<String>, GitError> {
    let working_directory_hash_list = get_hashes_working_directory(directory)?;
    let index_lines = get_lines_in_index(index_content.to_string());
    let index_hashes = get_hashes_index(index_lines)?;
    
    let deleted_files = check_for_deleted_files(&index_hashes, &working_directory_hash_list, directory);
    for file in deleted_files.iter() {
        formatted_result.push_str(&format!("{}\n", file));
    }

    Ok(deleted_files)
}

fn get_modified_files(directory: &str, index_content: &str, formatted_result: &mut String) {
    
}

fn get_other_files(directory: &str, index_content: &str, formatted_result: &mut String) {
    
}