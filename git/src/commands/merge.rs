use crate::commands::cat_file::git_cat_file;
use crate::errors::GitError;
use crate::models::client::Client;
use std::fs::File;
use std::io::{Read, Write};

use super::branch::get_current_branch;
use super::checkout::git_checkout_switch;
// use super::cat_file::git_cat_file;
use super::log::git_log;

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
    println!("current_branch: {}", current_branch);

    let path_current_branch = format!("{}/.git/refs/heads/{}", directory, current_branch);
    let path_branch_to_merge = format!("{}/.git/refs/heads/{}", directory, branch_name);

    println!("path_current_branch: {}", path_current_branch);
    println!("path_branch_to_merge: {}", path_branch_to_merge);

    let mut current_branch_file = match File::open(&path_current_branch){
        Ok(file) => file,
        Err(_) => return Err(GitError::OpenFileError),
    };
    let mut current_branch_hash: String = String::new();
    match current_branch_file.read_to_string(&mut current_branch_hash){
        Ok(file) => file,
        Err(_) => return Err(GitError::ReadFileError),
    };
    println!("current_branch_hash: {}", current_branch_hash);

    let mut merge_branch_file = match File::open(path_branch_to_merge){
        Ok(file) => file,
        Err(_) => return Err(GitError::OpenFileError),
    };
    let mut branch_to_merge_hash: String = String::new();
    match merge_branch_file.read_to_string(&mut branch_to_merge_hash){
        Ok(file) => file,
        Err(_) => return Err(GitError::ReadFileError),
    };
    println!("branch_to_merge_hash: {}", branch_to_merge_hash);

    if current_branch_hash == branch_to_merge_hash || current_branch_hash == branch_name{
        println!("Already up to date.");
        return Ok(());
    }


    // FORMATO LOG
    // Commit: 1a9bdce57b744e76f8fb7d8bbd0a272bbcd6e482
    // Author: Valen <vlanzillotta@fi.uba.ar>
    // Commit: 0a56b3675a740a84ad190edfacfecf17d0d4edea
    // Author: Juan <jdr@fi.uba.ar>


    else {
        let log_current_branch = git_log(directory)?;
        let log_current_branch = get_commits_from_log(log_current_branch);
        println!("log_current_branch: {:?}", log_current_branch);
        git_checkout_switch(directory, branch_name)?;
        println!("On branch {}", get_current_branch(directory)?);
        let log_merge_branch = git_log(directory)?;
        let log_merge_branch = get_commits_from_log(log_merge_branch);
        

        println!("log_merge_branch: {:?}", log_merge_branch);
        git_checkout_switch(directory, &current_branch)?;
        println!("On branch {}", get_current_branch(directory)?);
        
        for commit in log_current_branch.iter() {
            if let Some(last_hash_merge_branch) = log_merge_branch.last() {
                if commit == last_hash_merge_branch.as_str() {
                    println!("Already up to date.");
                    return Ok(());
                }
            }
        }
        // let last_hash_in_common = find_hash_in_common(&log_current_branch, &log_merge_branch)?;
        // println!("last_hash_in_common: {}", last_hash_in_common);
        // Actualizo la rama actual con su nuevo commit.
        let mut file_current_branch = match File::create(&path_current_branch){
            Ok(file) => file,
            Err(_) => return Err(GitError::CreateFileError),
        };
        match file_current_branch.write_all(branch_to_merge_hash.as_bytes()) {
            Ok(_) => (),
            Err(_) => return Err(GitError::WriteFileError),
        };
        println!("Updating {}..{}", current_branch_hash, branch_to_merge_hash);
        println!("Fast-forward");
        // let mut modified_files: Vec<&str> = Vec::new();
        let content_commit = git_cat_file(directory, &branch_to_merge_hash, "-p")?;
        println!("content_commit:\n{}", content_commit);
        for line in content_commit.lines() {
            if line.starts_with("tree ") {
                if let Some(hash) = line.strip_prefix("tree ") {
                    let hash_tree_in_commit = hash;
                    println!("hash_tree_in_commit: {}", hash_tree_in_commit);
                    let content_tree = git_cat_file(directory, hash_tree_in_commit, "-p")?;
                    println!("content_tree:\n{}", content_tree);
                }
            }
        }
        // let hash_tree_in_commit = content_commit.split(' ').collect::<Vec<&str>>()[1];
        // println!("hash_tree_in_commit: {}", hash_tree_in_commit);
        // let content_tree = git_cat_file(directory, hash_tree_in_commit, "-p")?;
        // println!("content_tree:\n{}", content_tree);
        // for line in content_tree.lines() {
        //     let file = line.split(' ').collect::<Vec<&str>>()[3];
        //     modified_files.push(file);
        // }
        // for file in modified_files {
        //     println!("{}", file);
        // }
        // me falta agregar los files de branch_to_merge en current_branch
    }

    Ok(())
}

/// Esta función se encarga de obtener los commits de un log
/// ###Parametros:
/// 'log': String que contiene el log
fn get_commits_from_log(log: String) -> Vec<String> {
    let mut commit_hashes: Vec<String> = Vec::new();

    for line in log.lines() {
        if line.starts_with("Commit: ") {
            if let Some(hash) = line.strip_prefix("Commit: ") {
                commit_hashes.push(hash.trim().to_string());
            }
        }
    }
    commit_hashes
}

/// Esta función se encarga de encontrar el hash en común entre dos logs de commits
/// ###Parametros:
/// 'log_current_branch': Vector de strings que contiene los hashes de los commits de la rama actual
/// 'log_merge_branch': Vector de strings que contiene los hashes de los commits de la rama a mergear
// fn find_hash_in_common(log_current_branch: &Vec<String>, log_merge_branch: &Vec<String>) -> Result<String, GitError> {
//     for commit_current in log_current_branch {
//         for commit_merge in log_merge_branch {
//             if commit_current == commit_merge {
//                 println!("Hash in common: {}", commit_current);
//                 return Ok(commit_current.to_string())
//             }
//         }
//     }
//     Err(GitError::HashInCommonNotFoundError)
// }

#[cfg(test)]
mod tests {
    use crate::commands::add::git_add;
    use crate::commands::branch::git_branch_create;
    use crate::commands::checkout::git_checkout_switch;
    use crate::commands::commit::Commit;
    use crate::commands::{init::git_init, commit::git_commit};
    use crate::commands::merge::git_merge;
    use std::{fs::{self}, io::Write};

    #[test]
    fn git_merge_test(){
        
        let directory = "./repo_test";
        git_init(directory).expect("Error al iniciar el repositorio");

        let file_path = format!("{}/{}", directory, "tocommitinmain.txt");
        let mut file = fs::File::create(&file_path).expect("Falló al crear el archivo");
        file.write_all(b"Archivo a commitear")
            .expect("Error al escribir en el archivo");

        let file_path2 = format!("{}/{}", directory, "tocommitinnew1.txt");
        let mut file2 = fs::File::create(&file_path2).expect("Falló al crear el archivo");
        file2.write_all(b"Otro archivo a commitear")
            .expect("Error al escribir en el archivo");


        let file_path3 = format!("{}/{}", directory, "tocommitinnew2.txt");
        let mut file = fs::File::create(&file_path3).expect("Falló al crear el archivo");
        file.write_all(b"Archivo a commitear 2")
            .expect("Error al escribir en el archivo");

        let file_path4 = format!("{}/{}", directory, "tocommitinnew3.txt");
        let mut file = fs::File::create(&file_path4).expect("Falló al crear el archivo");
        file.write_all(b"Archivo a commitear 3")
            .expect("Error al escribir en el archivo");

        git_add(directory, "tocommitinmain.txt").expect("Error al agregar el archivo");

        let test_commit1 = Commit::new(
            "prueba".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );
        git_commit(directory, test_commit1).expect("Error al commitear");

        git_branch_create(directory, "new_branch").expect("Error al crear la rama");
        
        git_checkout_switch(directory, "new_branch").expect("Error al cambiar de rama");

        git_add(directory, "tocommitinnew1.txt").expect("Error al agregar el archivo");

        let test_commit3 = Commit::new(
            "aa".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );

        git_commit(directory, test_commit3).expect("Error al commitear");

        git_add(directory, "tocommitinnew2.txt").expect("Error al agregar el archivo");

        let test_commit5 = Commit::new(
            "bb".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );

        git_commit(directory, test_commit5).expect("Error al commitear");

        git_add(directory, "tocommitinnew3.txt").expect("Error al agregar el archivo");

        let test_commit2 = Commit::new(
            "prueba otra".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );
        git_commit(directory, test_commit2).expect("Error al commitear");

        git_checkout_switch(directory, "main").expect("Error al cambiar de rama");
                
        git_merge(directory, "new_branch").expect("Error al mergear");

        fs::remove_dir_all(directory).expect("Falló al remover el directorio temporal");

        // assert_eq!(branch, "main");
    }

}