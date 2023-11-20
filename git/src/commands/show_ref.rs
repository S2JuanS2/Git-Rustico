use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::files::{open_file, read_file_string};
use std::fs;

/// Esta función se encarga de llamar a al comando ls-files con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función ls-files
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_show_ref(args: Vec<&str>, client: Client) -> Result<String, GitError> {
    if !args.is_empty() {
        return Err(GitError::InvalidArgumentShowRefError);
    }
    let directory = client.get_directory_path();
    git_show_ref(directory)
}

pub fn git_show_ref(directory: &str) -> Result<String, GitError> {

    let refs_heads_path = format!("{}/.git/refs/heads", directory);
    let refs_remotes_path = format!("{}/.git/refs/remotes", directory);
    let refs_tags_path = format!("{}/.git/refs/tags", directory);
    let mut formatted_result = String::new();

    visit_refs_dirs(refs_heads_path, &mut formatted_result)?;
    visit_refs_dirs(refs_remotes_path, &mut formatted_result)?;
    visit_refs_dirs(refs_tags_path, &mut formatted_result)?;

    Ok(formatted_result)
}

fn visit_refs_dirs(refs_path: String, formatted_result: &mut String) -> Result<(), GitError> {
    if fs::metadata(&refs_path).is_ok() {
        let mut entries = match fs::read_dir(&refs_path) {
            Ok(entries) => entries,
            Err(_) => return Err(GitError::ReadDirError),
        };
        while let Some(entry) = entries.next() {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => return Err(GitError::GenericError),
            };
            if let Ok(file_name) = entry.file_name().into_string() {
                let file_path = format!("{}/{}", refs_path, file_name);
                let file_hash = open_file(&file_path)?;
                let file_hash_content = read_file_string(file_hash)?;
                formatted_result.push_str(format!("{} {}/{}\n", file_hash_content.trim(), refs_path, file_name).as_str());
            }
        }
    }
    Ok(())
}