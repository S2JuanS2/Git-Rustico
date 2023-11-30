use crate::models::client::Client;
use crate::util::files::{create_file_replace, open_file, read_file_string};
use super::branch::get_current_branch;
use super::commit::{Commit, git_commit};
use super::errors::CommandsError;
use super::merge::{get_logs_from_branches, try_for_merge};
use super::cat_file::git_cat_file;

/// Esta función se encarga de llamar al comando rebase con los parametros necesarios.
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función rebase
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_rebase(args: Vec<&str>, client: Client) -> Result<String, CommandsError> {
    if args.len() != 1 {
        return Err(CommandsError::InvalidArgumentCountRebaseError);
    }
    let directory = client.get_directory_path();
    let branch_name = args[0];
    git_rebase(directory, branch_name, client.clone())
}

pub fn git_rebase(directory: &str, branch_name: &str, client: Client) -> Result<String, CommandsError> {
    let mut formatted_result = String::new();
    let current_branch = get_current_branch(directory)?;
    let (log_current_branch, log_rebase_branch) =
            get_logs_from_branches(directory, branch_name, &current_branch)?;

    formatted_result.push_str("First, rewinding head to replay your work on top of it...\n");
    let result_merge = try_for_merge(directory, branch_name)?;

    formatted_result.push_str(result_merge.as_str());
    if !result_merge.contains("CONFLICT") {

        let logs_just_in_current_branch = logs_just_in_one_branch(log_current_branch, log_rebase_branch);

        create_new_commits(directory, client, logs_just_in_current_branch, &current_branch, branch_name, &mut formatted_result)?;
        update_first_commit(directory, current_branch, branch_name)?;
    }

    Ok(formatted_result)
}

fn update_first_commit(directory: &str, current_branch: String, rebase_branch: &str) -> Result<(), CommandsError> {
    let (log_current_branch, log_rebase_branch) =
        get_logs_from_branches(directory, rebase_branch, &current_branch)?;

    let log_rebase_branch_cloned = log_rebase_branch.clone();
    let last_commit_rebase_branch = match log_rebase_branch_cloned.last() {
        Some(commit) => commit,
        None => return Err(CommandsError::GenericError), // CAMBIAR ERROR
    };

    let logs_just_in_current_branch = logs_just_in_one_branch(log_current_branch, log_rebase_branch);
    let first_commit_current_branch = &logs_just_in_current_branch[0];

    let content_commit = git_cat_file(directory, &first_commit_current_branch, "-p")?;

    update_parent(content_commit, last_commit_rebase_branch, &first_commit_current_branch, directory, current_branch)?;

    Ok(())
}

fn update_parent(content_commit: String, last_commit_rebase_branch: &String, first_commit_current_branch: &String, directory: &str, current_branch: String) -> Result<(), CommandsError> {
    let mut new_content = String::new();
    rewrite_commit(content_commit, last_commit_rebase_branch, &mut new_content);
    let lines_new_content: Vec<&str> = new_content.lines().collect();
    let new_commit_in_log = format!("{}\n{}\n{}\n{}\n\n{}", first_commit_current_branch, lines_new_content[1], lines_new_content[2], lines_new_content[3], lines_new_content[5]);
    let path_current_branch = format!("{}/.git/logs/refs/heads/{}", directory, current_branch);
    let file = open_file(&path_current_branch)?;
    let mut content = read_file_string(file)?;
    let lines: Vec<&str> = content.lines().collect();
    if let Some(index) = lines.iter().position(|&s| s == first_commit_current_branch) {
        let new_lines: Vec<&str> = new_commit_in_log.lines().collect();
        content = [
            &lines[..index],
            &new_lines,
            &lines[index + 6..],
        ]
        .concat()
        .join("\n");
    }
    create_file_replace(&path_current_branch, &content)?;
    Ok(())
}

fn rewrite_commit(content_commit: String, last_commit_rebase_branch: &String, new_content: &mut String) {
    for line in content_commit.lines() {
        if line.starts_with("parent") {
            let mut new_line = String::new();
            new_line.push_str(format!("parent {}", last_commit_rebase_branch).as_str());
            new_content.push_str(&new_line);
        } else {
            new_content.push_str(line);
        }
        new_content.push_str("\n");
    }
}

fn logs_just_in_one_branch(log_current_branch: Vec<String>, log_rebase_branch: Vec<String>) -> Vec<String> {
    let logs_just_in_current_branch = log_current_branch
        .iter()
        .filter(|commit| !log_rebase_branch.contains(commit))
        .collect::<Vec<_>>();
    logs_just_in_current_branch.iter().map(|commit| commit.to_string()).collect::<Vec<_>>()
}

fn create_new_commits(directory: &str, client: Client, log_current_branch: Vec<String>, current_branch: &str, branch_name: &str, formatted_result: &mut String) -> Result<(), CommandsError> {
    update_log_current_branch(directory, current_branch, branch_name)?;

    for commit in log_current_branch {
        let content_commit = git_cat_file(directory, &commit, "-p")?;
        let commit_message = get_commit_msg(content_commit);
        let new_commit = Commit::new(
            commit_message.clone(),
            client.get_name().to_string(),
            client.get_email().to_string(),
            client.get_name().to_string(),
            client.get_email().to_string(),
        );
        formatted_result.push_str(format!("Applying: {}\n", commit_message).as_str());
        git_commit(directory, new_commit)?;
    }
    Ok(())
}

fn update_log_current_branch(directory: &str, current_branch: &str, rebase_branch: &str) -> Result<(), CommandsError> {
    
    let path_rebase_log = format!("{}/.git/logs/refs/heads/{}", directory, rebase_branch);
    let file_rebase_log = open_file(&path_rebase_log)?;
    let content_rebase_log = read_file_string(file_rebase_log)?;
    let path_current_log = format!("{}/.git/logs/refs/heads/{}", directory, current_branch);

    create_file_replace(&path_current_log, &content_rebase_log)?;
    Ok(())
}

fn get_commit_msg(content_commit: String) -> String {
    let mut commit_message = String::new();
    for line in content_commit.lines() {
        if !line.starts_with("tree") && !line.starts_with("parent") && !line.starts_with("author") && !line.starts_with("committer") && !line.is_empty() {
            commit_message = line.to_string();
        }
    }
    commit_message
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::fs;

    use crate::commands::{init::git_init, add::git_add, commit::git_commit, checkout::git_checkout_switch, branch::git_branch_create, log::git_log};
    use super::*;
    
    #[test]
    fn test_rebase() {
        let directory = "./test_rebase";
        git_init(directory).expect("Error al inicializar el repositorio");

        let file_path = format!("{}/{}", directory, "holamundo.txt");
        let mut file = fs::File::create(&file_path).expect("Error al crear el archivo");
        file.write_all(b"Hola Mundo")
            .expect("Error al escribir en el archivo");

        let file_path2 = format!("{}/{}", directory, "holamundo2.txt");
        let mut file2 = fs::File::create(&file_path2).expect("Error al crear el archivo");
        file2.write_all(b"Hola Mundo 2")
            .expect("Error al escribir en el archivo");

        let file_path3 = format!("{}/{}", directory, "holamundo3.txt");
        let mut file3 = fs::File::create(&file_path3).expect("Error al crear el archivo");
        file3.write_all(b"Hola Mundo 3")
            .expect("Error al escribir en el archivo");

        git_add(directory, "holamundo.txt").expect("Error al hacer git add");

        let test_commit1 = Commit::new(
            "prueba".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );

        git_commit(directory, test_commit1).expect("Error al hacer git commit");

        git_branch_create(directory, "nueva_branch").expect("Error al crear la nueva branch");
        git_checkout_switch(directory, "nueva_branch").expect("Error al hacer git checkout");

        git_add(directory, "holamundo2.txt").expect("Error al hacer git add");

        let test_commit2 = Commit::new(
            "prueba otra".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );

        git_commit(directory, test_commit2).expect("Error al hacer git commit");

        git_add(directory, "holamundo3.txt").expect("Error al hacer git add");

        let test_commit3 = Commit::new(
            "aa".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );

        git_commit(directory, test_commit3).expect("Error al hacer git commit");

        let log_nueva_branch = git_log(directory).expect("Error al hacer git log");

        git_checkout_switch(directory, "master").expect("Error al hacer git checkout");

        let file_path2 = format!("{}/{}", directory, "holamundo2.txt");
        let mut file2 = fs::File::create(&file_path2).expect("Error al crear el archivo");
        file2.write_all(b"Hola Mundo 2")
            .expect("Error al escribir en el archivo");

        git_add(directory, "holamundo2.txt").expect("Error al hacer git add");

        let test_commit4 = Commit::new(
            "bb".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );

        git_commit(directory, test_commit4).expect("Error al hacer git commit");

        let client = Client::new(
            "Valen".to_string(), 
            "vlanzillotta@fi.uba.ar".to_string(),
            "19992020".to_string(),
            "9090".to_string(),
            "localhost".to_string(),
            "./".to_string(),
            "master".to_string(),
        );

        let result = git_rebase(directory, "nueva_branch", client);
        println!("sin conflicto:\n{:?}", result);

        let log_current_branch_after_rebase = git_log(directory).expect("Error al hacer git log");
        assert!(log_current_branch_after_rebase.contains(&log_nueva_branch));
        
        // fs::remove_dir_all(directory).expect("Error al borrar el directorio");
        assert!(result.is_ok());
    }

    #[test]
    fn test_rebase_with_conflict() {
        let directory = "./test_rebase_with_conflict";
        git_init(directory).expect("Error al inicializar el repositorio");

        let file_path = format!("{}/{}", directory, "holamundo.txt");
        let mut file = fs::File::create(&file_path).expect("Error al crear el archivo");
        file.write_all(b"Hola Mundo")
            .expect("Error al escribir en el archivo");

        let file_path2 = format!("{}/{}", directory, "holamundo2.txt");
        let mut file2 = fs::File::create(&file_path2).expect("Error al crear el archivo");
        file2.write_all(b"Hola Mundo 2")
            .expect("Error al escribir en el archivo");

        let file_path3 = format!("{}/{}", directory, "holamundo3.txt");
        let mut file3 = fs::File::create(&file_path3).expect("Error al crear el archivo");
        file3.write_all(b"Hola Mundo 3")
            .expect("Error al escribir en el archivo");

        git_add(directory, "holamundo.txt").expect("Error al hacer git add");

        let test_commit1 = Commit::new(
            "prueba".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );

        git_commit(directory, test_commit1).expect("Error al hacer git commit");

        git_branch_create(directory, "nueva_branch").expect("Error al crear la nueva branch");
        git_checkout_switch(directory, "nueva_branch").expect("Error al hacer git checkout");

        git_add(directory, "holamundo2.txt").expect("Error al hacer git add");

        let test_commit2 = Commit::new(
            "prueba otra".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );

        git_commit(directory, test_commit2).expect("Error al hacer git commit");

        git_add(directory, "holamundo3.txt").expect("Error al hacer git add");

        let test_commit3 = Commit::new(
            "aa".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );

        git_commit(directory, test_commit3).expect("Error al hacer git commit");

        let log_nueva_branch = git_log(directory).expect("Error al hacer git log");

        git_checkout_switch(directory, "master").expect("Error al hacer git checkout");

        // ESTA VEZ VA A HABER CONFLICTO EN HOLAMUNDO2.TXT
        let file_path2 = format!("{}/{}", directory, "holamundo2.txt");
        let mut file2 = fs::File::create(&file_path2).expect("Error al crear el archivo");
        file2.write_all(b"conflicto")
            .expect("Error al escribir en el archivo");

        git_add(directory, "holamundo2.txt").expect("Error al hacer git add");

        let test_commit4 = Commit::new(
            "bb".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );

        git_commit(directory, test_commit4).expect("Error al hacer git commit");

        let client = Client::new(
            "Valen".to_string(), 
            "vlanzillotta@fi.uba.ar".to_string(),
            "19992020".to_string(),
            "9090".to_string(),
            "localhost".to_string(),
            "./".to_string(),
            "master".to_string(),
        );

        let result = git_rebase(directory, "nueva_branch", client);
        println!("con conflicto:\n{:?}", result);

        let log_current_branch_after_rebase = git_log(directory).expect("Error al hacer git log");
        assert!(!log_current_branch_after_rebase.contains(&log_nueva_branch));
        
        // fs::remove_dir_all(directory).expect("Error al borrar el directorio");
        assert!(result.is_ok());
    }
}