use crate::errors::GitError;
use crate::models::client::Client;
use std::fs::File;
use std::io::Write;

use super::branch::get_current_branch;
// use super::checkout::git_checkout_switch;
use super::cat_file::git_cat_file;
// use super::log::git_log;

/// Esta función se encarga de llamar al comando merge con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función merge
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_merge(args: Vec<&str>, client: Client) -> Result<(), GitError> {
    if args.len() != 1 {
        return Err(GitError::InvalidArgumentCountMergeError);
    }
    let directory = client.get_directory_path();
    let branch_name = args[0];
    git_merge(directory, branch_name)
}

/// ejecuta la accion de merge en el repositorio local
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'branch_name': nombre de la rama a mergear
pub fn git_merge(directory: &str, branch_name: &str) -> Result<(), GitError> {
    let current_branch = get_current_branch(directory)?;

    let path_current_branch = format!("{}/.git/refs/heads/{}", directory, current_branch);
    let path_branch_to_merge = format!("{}/.git/refs/heads/{}", directory, branch_name);

    let current_branch_hash = match std::fs::read_to_string(path_current_branch){
        Ok(hash) => hash,
        Err(_) => return Err(GitError::ReadFileError),
    };
    let branch_to_merge_hash = match std::fs::read_to_string(path_branch_to_merge){
        Ok(hash) => hash,
        Err(_) => return Err(GitError::ReadFileError),
    };

    if current_branch_hash == branch_to_merge_hash || current_branch_hash == branch_name{
        println!("Already up to date.");
        return Ok(());
    }

    else {
        // let log_current_branch = git_log(directory)?;
        // git_checkout_switch(directory, branch_name)?;
        // let log_merge_branch = git_log(directory)?;
        // git_checkout_switch(directory, &current_branch)?;

        let log_current_branch: Vec<&str> = vec!["123456789", "98765432"];
        let log_merge_branch: Vec<&str> = vec!["123456789", "98765432", "56789876"];
        
        for commit in log_current_branch {
            let last_commit_merge_branch = match log_merge_branch.last() {
                Some(commit) => *commit,
                None => return Err(GitError::ReadFileError),
            };
            if last_commit_merge_branch == commit {
                println!("Already up to date.");
                return Ok(());
            }
        }
        // Actualizo la rama actual con su nuevo commit.
        let mut file_current_branch = match File::create(&current_branch_hash){
            Ok(file) => file,
            Err(_) => return Err(GitError::CreateFileError),
        };
        match file_current_branch.write_all(branch_to_merge_hash.as_bytes()) {
            Ok(_) => (),
            Err(_) => return Err(GitError::WriteFileError),
        };
        println!("Updating {}..{}", current_branch_hash, branch_to_merge_hash);
        println!("Fast-forward");
        let mut modified_files: Vec<&str> = Vec::new();
        let content_commit = git_cat_file(directory, &branch_to_merge_hash, "-p")?;
        let hash_tree_in_commit = content_commit.split(' ').collect::<Vec<&str>>()[1];
        let content_tree = git_cat_file(directory, hash_tree_in_commit, "-p")?;
        for line in content_tree.lines() {
            let file = line.split(' ').collect::<Vec<&str>>()[3];
            modified_files.push(file);
        }
        for file in modified_files {
            println!("{}", file);
        }
        // me falta agregar los files de branch_to_merge en current_branch
    }

    Ok(())
}
