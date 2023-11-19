use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::files::{open_file, read_file_string};
use std::fs;

use super::cat_file::git_cat_file;

/// Esta función se encarga de llamar a al comando ls-files con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función ls-files
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_ls_tree(args: Vec<&str>, client: Client) -> Result<String, GitError> {
    if args.len() > 1 {
        return Err(GitError::InvalidArgumentCountLsTreeError);
    }
    let directory = client.get_directory_path();
    git_ls_tree(directory, args[0])
}

pub fn git_ls_tree(directory: &str, tree_ish: &str) -> Result<String, GitError> {

    let mut tree_hash = tree_ish.to_string();
    let directory_tree = format!("{}/.git/{}", directory, tree_ish);
    if fs::metadata(&directory_tree).is_ok() {
        tree_hash = associated_commit(directory, &directory_tree)?;
    }
    if git_cat_file(directory, &tree_hash, "-t")? == "blob" {
        return Err(GitError::InvalidTreeHashError);
    }
    if git_cat_file(directory, &tree_hash, "-t")? == "commit"  {
        tree_hash = associated_tree(directory, tree_hash)?;
    }

    let mut formatted_result = String::new();
    let content_tree = git_cat_file(directory, &tree_hash, "-p")?;
    formatted_result.push_str(&format!("{}\n", content_tree));

    Ok(formatted_result)
}

fn associated_commit(directory: &str, path_to_commit: &str) -> Result<String, GitError> {
    let path = open_file(path_to_commit)?;
    let content = read_file_string(path)?;
    if content.len() != 40 {
        return Err(GitError::InvalidTreeHashError);
    }
    let tree = associated_tree(directory, content)?;

    Ok(tree)
}

fn associated_tree(directory: &str, content: String) -> Result<String, GitError> {
    let content_commit = git_cat_file(directory, &content, "-p")?;
    let parts: Vec<&str> = content_commit.split_whitespace().collect();
    let tree = parts[1];
    Ok(tree.to_string())
}

