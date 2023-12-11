use crate::consts::{FILE, GIT_DIR, CONTENT_EMPTY, PARENT_INITIAL};
use crate::models::client::Client;
use super::add::add_to_index;
use super::checkout::extract_parent_hash;
use super::commit::{Commit, git_commit};
use super::errors::CommandsError;
use super::rebase::rewrite_commit;
use crate::util::files::{create_file_replace, open_file, read_file_string};
use std::fs;
use std::path::Path;
use super::branch::get_current_branch;
use super::cat_file::git_cat_file;

/// Esta función se encarga de llamar al comando merge con los parametros necesarios.
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función merge
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_merge(args: Vec<&str>, client: Client) -> Result<String, CommandsError> {
    if args.len() != 1 {
        return Err(CommandsError::InvalidArgumentCountMergeError);
    }
    let directory = client.get_directory_path();
    let branch_name = args[0];
    let current_branch = get_current_branch(directory)?;
    git_merge(directory, &current_branch, branch_name, client.clone())
}

/// Ejecuta la accion de merge en el repositorio local.
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'branch_name': nombre de la rama a mergear
pub fn git_merge(directory: &str, current_branch: &str, branch_name: &str, client: Client) -> Result<String, CommandsError> {
    let formatted_result = try_for_merge(directory, current_branch, branch_name, &client, "merge")?;
    Ok(formatted_result)
}

/// Intenta fusionar la rama actual con otra rama pasada por parametro.
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'branch_name': nombre de la rama a mergear
pub fn try_for_merge(directory: &str, current_branch: &str, branch_name: &str, client: &Client, merge_type: &str) -> Result<String, CommandsError> {
    let path_current_branch = get_refs_path(directory, &current_branch);
    let path_branch_to_merge = get_refs_path(directory, branch_name);

    let (current_branch_hash, branch_to_merge_hash) = get_branches_hashes(&path_current_branch, &path_branch_to_merge)?;
    let mut formatted_result = String::new();
    
    if current_branch_hash == branch_to_merge_hash || current_branch_hash == branch_name {
        formatted_result.push_str("Already up to date.");
        return Ok(formatted_result);
    } else {
        let log_current_branch = get_log_from_branch(directory, &current_branch_hash)?;
        let log_merge_branch = get_log_from_branch(directory, &branch_to_merge_hash)?;
        if check_if_current_is_up_to_date(&log_current_branch, &log_merge_branch, &mut formatted_result) {
            return Ok(formatted_result);
        }

        let (first_commit_current_branch, first_commit_merge_branch) = get_first_commit_of_each_branch(&log_current_branch, &log_merge_branch);
        let root_parent_current_branch = git_cat_file(directory, &first_commit_current_branch, "-p")?;
        let root_parent_merge_branch = git_cat_file(directory, &first_commit_merge_branch, "-p")?;
        let (hash_parent_current, hash_parent_merge) = get_parent_hashes(root_parent_current_branch, root_parent_merge_branch);

        let strategy = merge_depending_on_strategy(&hash_parent_current, &hash_parent_merge, &branch_to_merge_hash, directory, branch_name)?;
        if merge_type == "merge" {
            update_refs(directory, &strategy, &path_current_branch, &branch_to_merge_hash, &path_branch_to_merge, client.clone())?;
            update_logs_refs(directory, &strategy, &current_branch, branch_name, &current_branch_hash, &branch_to_merge_hash)?;
        }
        get_result_depending_on_strategy(strategy, &mut formatted_result, current_branch_hash, branch_to_merge_hash, path_current_branch)?;
    }
    Ok(formatted_result)
}

/// Actualiza refs/heads/current_branch con el hash de la rama a mergear o, en caso de que la estrategia
/// que se utilizo sea recursive, se crea el merge commit y se actualizan los archivos con este commit.
/// ###Parametros:
/// 'strategy': tupla que contiene la estrategia de merge utilizada y el archivo en conflicto (u ok si no hay)
/// 'path_current_branch': path del refs/heads/current_branch
/// 'branch_to_merge_hash': hash del commit de la rama a mergear
fn update_refs(directory: &str, strategy: &(String, String), path_current_branch: &str, branch_to_merge_hash: &str, path_branch_to_merge: &str, client: Client) -> Result<(), CommandsError> {
    if strategy.0 == "recursive" && strategy.1 == "ok" {
        let merge_commit = Commit::new(
            "Merge Commit".to_string(),
            client.get_name().to_string(),
            client.get_email().to_string(),
            client.get_name().to_string(),
            client.get_email().to_string(),
        );
        git_commit(directory, merge_commit)?;
        update_merge_commit_in_other_branch(path_current_branch, path_branch_to_merge)?;
    } else if strategy.0 == "fast-forward" {
        create_file_replace(path_current_branch, branch_to_merge_hash)?;
    } else {
        return Ok(())
    }
    Ok(())
}

/// En el caso de que la estrategia de merge utilizada sea recursive, se actualiza el archivo de la rama a
/// mergear con el hash del merge commit.
/// ###Parametros:
/// 'path_current_branch': path del archivo de la rama actual
/// 'path_branch_to_merge': path del archivo de la rama a mergear
fn update_merge_commit_in_other_branch(path_current_branch: &str, path_branch_to_merge: &str) -> Result<(), CommandsError> {
    let current_branch_file = open_file(path_current_branch)?;
    let current_branch_hash = read_file_string(current_branch_file)?;
    create_file_replace(path_branch_to_merge, &current_branch_hash)?;

    Ok(())
}

/// En caso de que la estrategia de merge utilizada sea recursive, actualiza el archivo de logs de
/// la rama a mergear con el merge commit.
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'branch_name': nombre de la rama a mergear
fn update_logs_in_other_branch(directory: &str, branch_name: &str, branch_to_merge_hash: &str) -> Result<(), CommandsError> {
    let current_branch = get_current_branch(directory)?;
    let log_current_path = get_log_path(directory, &current_branch);
    let log_current_file = open_file(&log_current_path)?;
    let log_current_content = read_file_string(log_current_file)?;
    let last_commit_in_log_current = get_last_commit_in_log(log_current_content);

    let new_commit = rewrite_commit(&last_commit_in_log_current, branch_to_merge_hash);

    let log_branch_to_merge_path = get_log_path(directory, branch_name);
    let log_branch_to_merge_file = open_file(&log_branch_to_merge_path)?;
    let log_branch_to_merge_content = read_file_string(log_branch_to_merge_file)?;
    let log_branch_to_merge_content = format!("{}\n{}", log_branch_to_merge_content, new_commit);
    create_file_replace(&log_branch_to_merge_path, &log_branch_to_merge_content)?;

    Ok(())
}

/// Obtiene el path del archivo de una rama (en refs/heads si es local o en refs/remotes si es remota).
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'branch_name': nombre de la rama
pub fn get_refs_path(directory: &str, branch_name: &str) -> String {
    let mut path_branch_to_merge = format!("{}/.git/refs/heads/{}", directory, branch_name);
    if branch_name.contains("remotes") {
        path_branch_to_merge = format!("{}/.git/{}", directory, branch_name);
    }
    path_branch_to_merge
}

/// Obtiene el path del archivo de logs de una rama (en logs/refs/heads si es local o en logs/refs/remotes
/// si es remota).
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'branch_name': nombre de la rama
fn get_log_path(directory: &str, branch_name: &str) -> String {
    let mut log_path = format!("{}/.git/logs/refs/heads/{}", directory, branch_name);
    if branch_name.contains("remotes") {
        let branch_path = branch_name.split('/').collect::<Vec<_>>();
        if branch_path.len() >= 4 {
            log_path = format!("{}/.git/logs/refs/remotes/{}/{}", directory, branch_path[2], branch_path[3]);
        }else{
            log_path = format!("{}/.git/logs/refs/remotes/{}", directory, branch_path[2]);
        }
    }
    log_path
}

/// Actualiza el archivo de logs de la rama actual con los commits de la rama a mergear.
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'strategy': tupla que contiene la estrategia de merge utilizada y el archivo en conflicto (u ok si no hay)
/// 'current_branch': nombre de la rama actual
/// 'branch_to_merge': nombre de la rama a mergear
fn update_logs_refs(directory: &str, strategy: &(String, String), current_branch: &str, branch_to_merge: &str, current_branch_hash: &str, branch_to_merge_hash: &str) -> Result<(), CommandsError> {
    if strategy.0 == "recursive" && strategy.1 == "ok" {
        update_logs_in_other_branch(directory, branch_to_merge, branch_to_merge_hash)?;
    } else if strategy.0 == "fast-forward" {
        let logs_current_branch = get_log_from_branch(directory, &current_branch_hash)?;
        let logs_merge_branch = get_log_from_branch(directory, &branch_to_merge_hash)?;
        let logs_just_in_merge_branch = logs_just_in_one_branch(logs_merge_branch.to_vec(), logs_current_branch.to_vec());
        // revertir el orden de logs_just_in_merge_branch
        let logs_just_in_merge_branch = logs_just_in_merge_branch.iter().rev().collect::<Vec<_>>();
        let log_merge_path = get_log_path(directory, branch_to_merge);
        let log_merge_file = open_file(&log_merge_path)?;
        let log_merge_content = read_file_string(log_merge_file)?;

        let new_commits: String = log_merge_content
            .lines()
            .skip_while(|&line| !line.contains(logs_just_in_merge_branch[0]))
            .collect::<Vec<&str>>()
            .join("\n");

        let log_current_path = get_log_path(directory, current_branch);
        let log_current_file = open_file(&log_current_path)?;
        let mut log_current_content = read_file_string(log_current_file)?;
        log_current_content.push_str(format!("\n{}", new_commits).as_str());
        create_file_replace(&log_current_path, &log_current_content)?;
    } else {
        return Ok(())
    }
    Ok(())
}

/// Obtiene el ultimo commit del log con su informacion.
/// ###Parametros:
/// 'log': String que contiene el log
fn get_last_commit_in_log(log: String) -> String {
    let mut last_commit = String::new();
    let total_lines = log.lines().count();
    for (count, line) in log.lines().enumerate() {
        if count >= (total_lines - 6) {
            if count == (total_lines - 1) {
                last_commit.push_str(line);
            } else {
                last_commit.push_str(line);
                last_commit.push('\n');
            }
        }
    }
    last_commit
}

/// Obtiene los logs que difieren entre las ramas a mergear.
/// ###Parametros:
/// 'log_current_branch': logs de la branch actual
/// 'log_other_branch': logs de otra branch
pub fn logs_just_in_one_branch(log_current_branch: Vec<String>, log_other_branch: Vec<String>) -> Vec<String> {
    let logs_just_in_current_branch = log_current_branch
        .iter()
        .filter(|commit| !log_other_branch.contains(commit))
        .collect::<Vec<_>>();
    logs_just_in_current_branch.iter().map(|commit| commit.to_string()).collect::<Vec<_>>()
}

/// Obtiene el primer commit de cada rama por separado.
/// ###Parametros:
/// 'log_current_branch': Vector de strings que contiene los commits de la rama actual.
/// 'log_merge_branch': Vector de strings que contiene los commits de la rama a mergear.
pub fn get_first_commit_of_each_branch(
    log_current_branch: &[String],
    log_merge_branch: &[String],
) -> (String, String) {

    let logs_just_in_current_branch = logs_just_in_one_branch(log_current_branch.to_vec(), log_merge_branch.to_vec());
    let logs_just_in_merge_branch = logs_just_in_one_branch(log_merge_branch.to_vec(), log_current_branch.to_vec());

    let mut first_commit_current_branch = &log_current_branch[0];
    let mut first_commit_merge_branch = &log_merge_branch[0];

    if !logs_just_in_current_branch.is_empty() {
        first_commit_current_branch = match &logs_just_in_current_branch.last() {
            Some(commit) => commit,
            None => first_commit_current_branch,
        }
    }

    if !logs_just_in_merge_branch.is_empty() {
        first_commit_merge_branch = match &logs_just_in_merge_branch.last() {
            Some(commit) => commit,
            None => first_commit_merge_branch,
        }
    }
    (
        first_commit_current_branch.to_string(),
        first_commit_merge_branch.to_string(),
    )
}

/// Obtiene los hashes de los commits de las ramas a mergear.
/// ###Parametros:
/// 'path_current_branch': path del archivo de la rama actual
/// 'path_branch_to_merge': path del archivo de la rama a mergear
pub fn get_branches_hashes(
    path_current_branch: &str,
    path_branch_to_merge: &str,
) -> Result<(String, String), CommandsError> {
    let current_branch_file = open_file(path_current_branch)?;
    let current_branch_hash = read_file_string(current_branch_file)?;
    let merge_branch_file = open_file(path_branch_to_merge)?;
    let branch_to_merge_hash = read_file_string(merge_branch_file)?;
    Ok((current_branch_hash, branch_to_merge_hash))
}

/// Obtiene el log de la rama pasada por parametro.
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'branch': nombre de la rama
pub fn get_log_from_branch(
    directory: &str,
    hash: &str,
) -> Result<Vec<String>, CommandsError> {

    let mut log_current_branch: Vec<String> = Vec::new();
    let commit_content = git_cat_file(directory, hash, "-p")?;
    add_commit_to_log(directory, &mut log_current_branch, hash, commit_content)?;
    
    Ok(log_current_branch)
}

fn add_commit_to_log(directory: &str, log_current_branch: &mut Vec<String>, hash: &str, commit_content: String) -> Result<(), CommandsError> {
    log_current_branch.push(hash.to_string());
    if let Some(parent_hash) = extract_parent_hash(&commit_content) {
        let commit_content = git_cat_file(directory, parent_hash, "-p")?;
        add_commit_to_log(directory, log_current_branch, parent_hash, commit_content)?;
    }
    Ok(())
}

/// Obtiene el resultado del merge dependiendo de la estrategia de merge utilizada.
/// ###Parametros:
/// 'strategy': tupla que contiene la estrategia de merge utilizada y el archivo en conflicto (u ok si no hay
/// conflictos)
/// 'formatted_result': String que contiene el resultado formateado del merge.
/// 'current_branch_hash': hash del commit de la rama actual
/// 'branch_to_merge_hash': hash del commit de la rama a mergear
/// 'path_current_branch': path del archivo de la rama actual
fn get_result_depending_on_strategy(
    strategy: (String, String),
    formatted_result: &mut String,
    current_branch_hash: String,
    branch_to_merge_hash: String,
    path_current_branch: String,
) -> Result<(), CommandsError> {
    if strategy.0 == "recursive" && strategy.1 == "ok" {
        formatted_result.push_str("Merge made by the 'recursive' strategy.");
    } else if strategy.0 == "fast-forward" {
        formatted_result.push_str(
            format!(
                "Updating {}..{}\n",
                current_branch_hash, branch_to_merge_hash
            )
            .as_str(),
        );
        formatted_result.push_str("Fast-forward\n");
        create_file_replace(&path_current_branch, &branch_to_merge_hash)?;
    } else {
        formatted_result.push_str(format!("Auto-merging {}\n", strategy.1).as_str());
        formatted_result.push_str(
            format!(
                "CONFLICT (content): Merge conflict in {}\n",
                strategy.1
            )
            .as_str(),
        );
        formatted_result
            .push_str("Automatic merge failed; fix conflicts and then commit the result.\n");
        formatted_result.push_str(format!("Conflict in file:{}\n", strategy.1).as_str());
    }
    Ok(())
}

/// Obtiene los hashes de los padres de los commits de las ramas a mergear.
/// ###Parametros:
/// 'root_parent_current_branch': String que contiene el contenido del primer commit de la rama actual
/// 'root_parent_merge_branch': String que contiene el contenido del primer commit de la rama a mergear
pub fn get_parent_hashes(
    root_parent_current_branch: String,
    root_parent_merge_branch: String,
) -> (String, String) {
    let mut hash_parent_current = PARENT_INITIAL;
    let mut hash_parent_merge = PARENT_INITIAL;
    for line in root_parent_current_branch.lines() {
        if line.starts_with("parent ") {
            if let Some(hash) = line.strip_prefix("parent ") {
                hash_parent_current = hash;
            }
        }
    }
    for line in root_parent_merge_branch.lines() {
        if line.starts_with("parent ") {
            if let Some(hash) = line.strip_prefix("parent ") {
                hash_parent_merge = hash;
            }
        }
    }
    (
        hash_parent_current.to_string(),
        hash_parent_merge.to_string(),
    )
}

/// Obtiene el path del archivo en conflicto.
/// ###Parametros:
/// 'conflict_msg': String que contiene el mensaje de conflicto
pub fn get_conflict_path(conflict_msg: &str) -> String {
    let mut conflict_path = String::new();
    for line in conflict_msg.lines() {
        if line.starts_with("CONFLICT (content): Merge conflict in ") {
            if let Some(path) = line.strip_prefix("CONFLICT (content): Merge conflict in ") {
                conflict_path = path.to_string();
            }
        }
    }
    conflict_path
}

/// Chequea si la rama actual tiene como commit al ultimo commit de la rama a mergear. En caso de tenerlo,
/// la rama actual esta mas avanzada que la rama a mergear entonces estaria actualizada.
/// ###Parametros:
/// 'log_current_branch': Vector de strings que contiene los commits de la rama actual.
/// 'log_merge_branch': Vector de strings que contiene los commits de la rama a mergear.
/// 'formatted_result': String que contiene el resultado formateado del merge.
pub fn check_if_current_is_up_to_date(
    log_current_branch: &[String],
    log_merge_branch: &[String],
    formatted_result: &mut String,
) -> bool {
    for commit in log_current_branch.iter() {
        if commit == log_merge_branch[0].as_str() {
            formatted_result.push_str("Already up to date.");
            return true;
        }
    }
    false
}

/// Recorre los tree en la merge branch y hace el merge dependiendo de la estrategia a utilizar.
/// ###Parametros:
/// 'hash_parent_current': hash del padre del commit de la rama actual
/// 'hash_parent_merge': hash del padre del commit de la rama a mergear
/// 'log_merge_branch': hash del tree
/// 'directory': directorio del repositorio local
/// 'branch_to_merge': nombre de la rama a mergear
fn recovery_tree_merge(
    directory: &str,
    hash_parent_current: &str,
    hash_parent_merge: &str,
    branch_to_merge: &str,
    content_tree: String,
    path: &str)
    -> Result<(String, String), CommandsError>{
    let mut strategy: (String, String) = ("".to_string(), "".to_string());
    for line in content_tree.lines() {
        let file = line.split_whitespace().skip(1).take(1).collect::<String>();
        let mode = line.split_whitespace().take(1).collect::<String>();
        let hash_object = line.split_whitespace().skip(2).take(1).collect::<String>();
        let content_file = git_cat_file(directory, &hash_object, "-p")?;
        let path_file_format = format!("{}/{}{}", directory, path, file);
        if mode == FILE {
            let path_file_format_clean = Path::new(&path_file_format).strip_prefix(directory).unwrap();
            let path_file_format_clean_str = path_file_format_clean.to_str().ok_or(CommandsError::PathToStringError)?;
            let git_dir = format!("{}/{}", directory, GIT_DIR);
            if hash_parent_current == hash_parent_merge {
                // RECURSIVE STRATEGY
                if let Ok(metadata) = fs::metadata(&path_file_format) {
                    if metadata.is_file() {
                        compare_files(
                            &path_file_format,
                            &content_file,
                            branch_to_merge,
                            &mut strategy,
                        )?;
                        if strategy.0 == "recursive" && strategy.1 != "ok" {
                            return Ok(strategy);
                        }
                    }
                } else {
                    add_to_index(git_dir, path_file_format_clean_str, hash_object)?;
                    create_file_replace(&path_file_format, &content_file)?;
                    strategy.0 = "recursive".to_string();
                    strategy.1 = "ok".to_string();
                };
            } else {
                // FAST-FORWARD STRATEGY
                create_file_replace(&path_file_format, &content_file)?;
                add_to_index(git_dir, path_file_format_clean_str, hash_object)?;
                strategy.0 = "fast-forward".to_string();
                strategy.1 = "ok".to_string();
            }
        }else{
            let path = format!("{}{}/", path, file);
            let content_tree = git_cat_file(directory, &hash_object, "-p")?;
            recovery_tree_merge(directory,
                hash_parent_current, hash_parent_merge,
                branch_to_merge,
                content_tree,
                &path
            )?;
        }
    }
    Ok(strategy)
}

/// Recorre los commits en la merge branch y hace el merge dependiendo de la estrategia a utilizar.
/// ###Parametros:
/// 'hash_parent_current': hash del padre del commit de la rama actual
/// 'hash_parent_merge': hash del padre del commit de la rama a mergear
/// 'log_merge_branch': hash del ultimo commit
/// 'directory': directorio del repositorio local
/// 'branch_to_merge': nombre de la rama a mergear
pub fn merge_depending_on_strategy(
    hash_parent_current: &str,
    hash_parent_merge: &str,
    branch_to_merge_hash: &str,
    directory: &str,
    branch_to_merge: &str,
) -> Result<(String, String), CommandsError> {
    let content_commit = git_cat_file(directory, branch_to_merge_hash, "-p")?;
    let content_tree = get_tree_of_commit(content_commit, directory)?;
    let strategy = recovery_tree_merge(
        directory,
        hash_parent_current,
        hash_parent_merge,
        branch_to_merge,
        content_tree,
        CONTENT_EMPTY)?;
    Ok(strategy)
}

/// Obtiene el contenido del arbol del commit.
/// ###Parametros:
/// 'content_commit': String que contiene el contenido del commit
/// 'directory': directorio del repositorio local
pub fn get_tree_of_commit(content_commit: String, directory: &str) -> Result<String, CommandsError> {
    let mut content_tree = String::new();
    for line in content_commit.lines() {
        if line.starts_with("tree ") {
            if let Some(hash) = line.strip_prefix("tree ") {
                let hash_tree_in_commit = hash;
                content_tree = git_cat_file(directory, hash_tree_in_commit, "-p")?;
            }
        }
    }
    Ok(content_tree)
}

/// Compara el archivo en la rama actual con el de la rama a mergear.
/// ###Parametros:
/// 'path_file_format': path del archivo a comparar
/// 'content_file': contenido del archivo en la rama a mergear
/// 'branch_to_merge': nombre de la rama a mergear
/// 'strategy': tupla que contiene la estrategia de merge utilizada y el archivo en conflicto (u ok si no hay)
fn compare_files(
    path_file_format: &str,
    content_file: &str,
    branch_to_merge: &str,
    strategy: &mut (String, String),
) -> Result<(), CommandsError> {
    let file = open_file(path_file_format)?;
    let content_file_local = read_file_string(file)?;
    if content_file_local != content_file {
        // CONFLICTO
        let path_conflict = check_each_line(
            path_file_format,
            content_file_local,
            content_file,
            branch_to_merge,
        )?;
        strategy.0 = "recursive".to_string();
        strategy.1 = path_conflict;
    } else {
        // NO CONFLICTO
        create_file_replace(path_file_format, content_file)?;
        strategy.0 = "recursive".to_string();
        strategy.1 = "ok".to_string();
    }
    Ok(())
}

/// Esta función se encarga de obtener los commits de un log.
/// ###Parametros:
/// 'log': String que contiene el log
pub fn get_commits_from_log(log: String) -> Vec<String> {
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

/// Chequea cada linea de los archivos para ver si hay conflictos.
/// ###Parametros:
/// 'path_file_format': path del archivo a comparar
/// 'content_file_local': contenido del archivo en la rama actual
/// 'content_file': contenido del archivo en la rama a mergear
/// 'branch_to_merge': nombre de la rama a mergear
fn check_each_line(
    path_file_format: &str,
    content_file_local: String,
    content_file: &str,
    branch_to_merge: &str,
) -> Result<String, CommandsError> {
    let mut content_file_local_lines = content_file_local.lines();
    let mut content_file_lines = content_file.lines();
    let mut line_local = content_file_local_lines.next();
    let mut line = content_file_lines.next();

    let mut new_content_file = String::new();

    while line_local.is_some() && line.is_some() {
        if line_local != line {
            new_content_file.push_str("<<<<<<< HEAD\n");
            if let Some(line_local) = line_local {
                new_content_file.push_str(line_local);
            }
            new_content_file.push_str("\n=======\n");
            if let Some(line) = line {
                new_content_file.push_str(line);
            }
            new_content_file.push_str("\n>>>>>>> ");
            new_content_file.push_str(branch_to_merge);
            new_content_file.push('\n');
        } else {
            if let Some(line_local) = line_local {
                new_content_file.push_str(line_local);
            }
            new_content_file.push('\n');
        }
        line_local = content_file_local_lines.next();
        line = content_file_lines.next();
    }
    create_file_replace(path_file_format, &new_content_file)?;
    Ok(path_file_format.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::add::git_add;
    use crate::commands::branch::git_branch_create;
    use crate::commands::checkout::git_checkout_switch;
    use crate::commands::commit::Commit;
    use crate::commands::merge::git_merge;
    use crate::commands::{commit::git_commit, init::git_init};
    use crate::util::files::{create_file_replace, open_file, read_file_string};
    use std::fs::create_dir_all;
    use std::{
        fs::{self},
        io::Write,
    };

    #[test]
    fn git_merge_fast_forward_test() {
        let directory = "./test_merge_fast_forward";
        git_init(directory).expect("Error al iniciar el repositorio");

        let file_path = format!("{}/{}", directory, "tocommitinmain.txt");
        let mut file = fs::File::create(&file_path).expect("Falló al crear el archivo");
        file.write_all(b"Archivo a commitear")
            .expect("Error al escribir en el archivo");

        let file_path2 = format!("{}/{}", directory, "tocommitinnew1.txt");
        let mut file2 = fs::File::create(&file_path2).expect("Falló al crear el archivo");
        file2
            .write_all(b"Otro archivo a commitear")
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

        let test_commit2 = Commit::new(
            "prueba otra".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );

        git_commit(directory, test_commit1).expect("Error al commitear");

        git_add(directory, "tocommitinnew3.txt").expect("Error al agregar el archivo");

        git_commit(directory, test_commit2).expect("Error al commitear");

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

        git_checkout_switch(directory, "master").expect("Error al cambiar de rama");

        let client: Client = Client::new(
            "Valen".to_string(), 
            "vlanzillotta@fi.uba.ar".to_string(),
            "19992020".to_string(),
            "9090".to_string(),
            "localhost".to_string(),
            "./".to_string(),
            "master".to_string(),
        );

        let result = git_merge(directory, "master", "new_branch", client);

        fs::remove_dir_all(directory).expect("Falló al remover el directorio temporal");

        assert!(result.is_ok());
    }

    #[test]
    fn git_merge_recursive_with_conflict_test() {
        let directory = "./test_merge_recursive";
        git_init(directory).expect("Error al iniciar el repositorio");

        let file_path = format!("{}/{}", directory, "tocommitinmaster.txt");
        let mut file = fs::File::create(&file_path).expect("Falló al crear el archivo");
        file.write_all(b"Archivo a commitear")
            .expect("Error al escribir en el archivo");

        git_add(directory, "tocommitinmaster.txt").expect("Error al agregar el archivo");

        let test_commit1 = Commit::new(
            "prueba".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );
        git_commit(directory, test_commit1).expect("Error al commitear");

        git_branch_create(directory, "new_branch").expect("Error al crear la rama");

        git_branch_create(directory, "otra_mas").expect("Error al crear la rama");

        git_checkout_switch(directory, "new_branch").expect("Error al cambiar de rama");

        let file_path2 = format!("{}/{}", directory, "tocommitinnew1.txt");
        let mut file2 = fs::File::create(&file_path2).expect("Falló al crear el archivo");
        file2
            .write_all(b"Otro archivo a commitear")
            .expect("Error al escribir en el archivo");

        let file_path3 = format!("{}/{}", directory, "tocommitinnew2.txt");
        let mut file = fs::File::create(&file_path3).expect("Falló al crear el archivo");
        file.write_all(b"Archivo a commitear 2")
            .expect("Error al escribir en el archivo");

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

        git_checkout_switch(directory, "otra_mas").expect("Error al cambiar de rama");

        let file_path4 = format!("{}/{}", directory, "tocommitinnew2.txt");
        let mut file = fs::File::create(&file_path4).expect("Falló al crear el archivo");
        file.write_all(b"Archivo a commitear 32")
            .expect("Error al escribir en el archivo");

        git_add(directory, "tocommitinnew2.txt").expect("Error al agregar el archivo");

        let test_commit2 = Commit::new(
            "prueba otra".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );
        git_commit(directory, test_commit2).expect("Error al commitear");

        let client: Client = Client::new(
            "Valen".to_string(), 
            "vlanzillotta@fi.uba.ar".to_string(),
            "19992020".to_string(),
            "9090".to_string(),
            "localhost".to_string(),
            "./".to_string(),
            "master".to_string(),
        );

        let result = git_merge(directory, "otra_mas", "new_branch", client);

        fs::remove_dir_all(directory).expect("Falló al remover el directorio temporal");

        assert!(result.is_ok());
    }

    #[test]
    fn git_merge_recursive_without_conflict_test() {
        let directory = "./test_merge_recursive_without_conflict";
        git_init(directory).expect("Error al iniciar el repositorio");

        let file_path = format!("{}/{}", directory, "tocommitinmaster.txt");
        let mut file = fs::File::create(&file_path).expect("Falló al crear el archivo");
        file.write_all(b"Archivo a commitear")
            .expect("Error al escribir en el archivo");

        let file_path2 = format!("{}/{}", directory, "tocommitinnew1.txt");
        let mut file2 = fs::File::create(&file_path2).expect("Falló al crear el archivo");
        file2
            .write_all(b"Otro archivo a commitear")
            .expect("Error al escribir en el archivo");

        let file_path3 = format!("{}/{}", directory, "tocommitinnew2.txt");
        let mut file = fs::File::create(&file_path3).expect("Falló al crear el archivo");
        file.write_all(b"Archivo a commitear 2")
            .expect("Error al escribir en el archivo");

        git_add(directory, "tocommitinmaster.txt").expect("Error al agregar el archivo");

        let test_commit1 = Commit::new(
            "prueba".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );
        git_commit(directory, test_commit1).expect("Error al commitear");

        git_branch_create(directory, "new_branch").expect("Error al crear la rama");

        git_branch_create(directory, "otra_mas").expect("Error al crear la rama");

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

        git_checkout_switch(directory, "otra_mas").expect("Error al cambiar de rama");

        let file_path4 = format!("{}/{}", directory, "tocommitinnew2.txt");
        let mut file = fs::File::create(&file_path4).expect("Falló al crear el archivo");
        file.write_all(b"Archivo a commitear 2")
            .expect("Error al escribir en el archivo");

        git_add(directory, "tocommitinnew2.txt").expect("Error al agregar el archivo");

        let test_commit2 = Commit::new(
            "prueba otra".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );
        git_commit(directory, test_commit2).expect("Error al commitear");

        let client: Client = Client::new(
            "Valen".to_string(), 
            "vlanzillotta@fi.uba.ar".to_string(),
            "19992020".to_string(),
            "9090".to_string(),
            "localhost".to_string(),
            "./".to_string(),
            "master".to_string(),
        );

        let result = git_merge(directory, "otra_mas", "new_branch", client);
        
        fs::remove_dir_all(directory).expect("Falló al remover el directorio temporal");

        assert!(result.is_ok());
    }

    #[test]
    fn git_merge_local_and_remote_paths() {
        let directory = "./test_merge_local_and_remote_paths";
        git_init(directory).expect("Error al iniciar el repositorio");

        let file_path = format!("{}/{}", directory, "tocommitinmaster.txt");
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

        git_add(directory, "tocommitinmaster.txt").expect("Error al agregar el archivo");

        let test_commit1 = Commit::new(
            "prueba".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );
        git_commit(directory, test_commit1).expect("Error al commitear");

        // para copiar el commit a una branch remota
        git_branch_create(directory, "new_branch").expect("Error al crear la rama");
        git_checkout_switch(directory, "new_branch").expect("Error al cambiar de rama");

        git_add(directory, "tocommitinnew1.txt").expect("Error al agregar el archivo");
        git_add(directory, "tocommitinnew2.txt").expect("Error al agregar el archivo");

        let test_commit2 = Commit::new(
            "para remote".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );

        git_commit(directory, test_commit2).expect("Error al commitear");

        let new_branch_path = format!("{}/.git/refs/heads/{}", directory, "new_branch");
        let new_branch_file = open_file(&new_branch_path).expect("Error al abrir el archivo");
        let new_branch_hash = read_file_string(new_branch_file).expect("Error al leer el archivo");

        create_dir_all(format!("{}/.git/logs", directory)).expect("Error al crear el directorio");
        create_dir_all(format!("{}/.git/logs/refs", directory)).expect("Error al crear el directorio");
        create_dir_all(format!("{}/.git/logs/refs/remotes", directory)).expect("Error al crear el directorio");
        create_dir_all(format!("{}/.git/logs/refs/remotes/origin", directory)).expect("Error al crear el directorio");

        let log_new = open_file(format!("{}/.git/logs/refs/heads/new_branch", directory).as_str()).expect("Error al abrir el archivo");
        let log_new_content = read_file_string(log_new).expect("Error al leer el archivo");

        let log_remote = format!("{}/.git/logs/refs/remotes/origin/fetch", directory);
        create_file_replace(&log_remote, &log_new_content).expect("Error al crear el archivo");

        let remote_branch_path = format!("{}/.git/refs/remotes/origin/fetch", directory);
        create_file_replace(&remote_branch_path, &new_branch_hash).expect("Error al crear el archivo");

        git_checkout_switch(directory, "master").expect("Error al cambiar de rama");

        let client: Client = Client::new(
            "Valen".to_string(), 
            "vlanzillotta@fi.uba.ar".to_string(),
            "19992020".to_string(),
            "9090".to_string(),
            "localhost".to_string(),
            "./".to_string(),
            "master".to_string(),
        );
        
        // para confirmar que busca en refs/remotes y no en refs/heads
        fs::remove_file(new_branch_path).expect("Error al remover el archivo");
        let log_path_local = format!("{}/.git/logs/refs/heads/new_branch", directory);
        fs::remove_file(log_path_local).expect("Error al remover el archivo");

        let result = git_merge(directory, "master", "refs/remotes/origin/fetch", client);

        fs::remove_dir_all(directory).expect("Falló al remover el directorio temporal");
        assert!(result.is_ok());
    }

    #[test]
    fn git_merge_local_and_remote_paths_conflict() {
        let directory = "./test_merge_local_and_remote_paths_conflict";
        git_init(directory).expect("Error al iniciar el repositorio");

        let file_path = format!("{}/{}", directory, "tocommitinmaster.txt");
        let mut file = fs::File::create(&file_path).expect("Falló al crear el archivo");
        file.write_all(b"Archivo a commitear")
            .expect("Error al escribir en el archivo");

        let file_path2 = format!("{}/{}", directory, "forconflict.txt");
        let mut file2 = fs::File::create(&file_path2).expect("Falló al crear el archivo");
        file2.write_all(b"Otro archivo a commitear")
            .expect("Error al escribir en el archivo");

        git_add(directory, "tocommitinmaster.txt").expect("Error al agregar el archivo");

        let test_commit1 = Commit::new(
            "prueba".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );
        git_commit(directory, test_commit1).expect("Error al commitear");

        // para copiar el commit a una branch remota
        git_branch_create(directory, "new_branch").expect("Error al crear la rama");
        git_checkout_switch(directory, "new_branch").expect("Error al cambiar de rama");

        git_add(directory, "forconflict.txt").expect("Error al agregar el archivo");

        let test_commit2 = Commit::new(
            "para remote".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );

        git_commit(directory, test_commit2).expect("Error al commitear");

        git_checkout_switch(directory, "master").expect("Error al cambiar de rama");

        let file_path3 = format!("{}/{}", directory, "forconflict.txt");
        let mut file = fs::File::create(&file_path3).expect("Falló al crear el archivo");
        file.write_all(b"Para conflicto")
            .expect("Error al escribir en el archivo");

        git_add(directory, "forconflict.txt").expect("Error al agregar el archivo");

        let test_commit3 = Commit::new(
            "para conflicto".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );

        git_commit(directory, test_commit3).expect("Error al commitear");

        let new_branch_path = format!("{}/.git/refs/heads/{}", directory, "new_branch");
        let new_branch_file = open_file(&new_branch_path).expect("Error al abrir el archivo");
        let new_branch_hash = read_file_string(new_branch_file).expect("Error al leer el archivo");

        create_dir_all(format!("{}/.git/logs", directory)).expect("Error al crear el directorio");
        create_dir_all(format!("{}/.git/logs/refs", directory)).expect("Error al crear el directorio");
        create_dir_all(format!("{}/.git/logs/refs/remotes", directory)).expect("Error al crear el directorio");
        create_dir_all(format!("{}/.git/logs/refs/remotes/origin", directory)).expect("Error al crear el directorio");

        let log_new = open_file(format!("{}/.git/logs/refs/heads/new_branch", directory).as_str()).expect("Error al abrir el archivo");
        let log_new_content = read_file_string(log_new).expect("Error al leer el archivo");

        let log_remote = format!("{}/.git/logs/refs/remotes/origin/fetch", directory);
        create_file_replace(&log_remote, &log_new_content).expect("Error al crear el archivo");

        let remote_branch_path = format!("{}/.git/refs/remotes/origin/fetch", directory);
        create_file_replace(&remote_branch_path, &new_branch_hash).expect("Error al crear el archivo");

        let client: Client = Client::new(
            "Valen".to_string(), 
            "vlanzillotta@fi.uba.ar".to_string(),
            "19992020".to_string(),
            "9090".to_string(),
            "localhost".to_string(),
            "./".to_string(),
            "master".to_string(),
        );

        // para confirmar que busca en refs/remotes y no en refs/heads
        fs::remove_file(new_branch_path).expect("Error al remover el archivo");
        let log_path_local = format!("{}/.git/logs/refs/heads/new_branch", directory);
        fs::remove_file(log_path_local).expect("Error al remover el archivo");

        let result = git_merge(directory, "master", "refs/remotes/origin/fetch", client);

        fs::remove_dir_all(directory).expect("Falló al remover el directorio temporal");
        assert!(result.is_ok());
    }

    #[test]
    fn git_merge_remote_and_local_paths() {
        let directory = "./test_merge_remote_and_local_paths";
        git_init(directory).expect("Error al iniciar el repositorio");

        let file_path = format!("{}/{}", directory, "tocommitinmaster.txt");
        let mut file = fs::File::create(&file_path).expect("Falló al crear el archivo");
        file.write_all(b"Archivo a commitear")
            .expect("Error al escribir en el archivo");

        let file_path2 = format!("{}/{}", directory, "tocommitonlyonmaster1.txt");
        let mut file2 = fs::File::create(&file_path2).expect("Falló al crear el archivo");
        file2.write_all(b"Otro archivo a commitear")
            .expect("Error al escribir en el archivo");

        let file_path3 = format!("{}/{}", directory, "tocommitonlyonmaster2.txt");
        let mut file = fs::File::create(&file_path3).expect("Falló al crear el archivo");
        file.write_all(b"Archivo a commitear 2")
            .expect("Error al escribir en el archivo");

        git_add(directory, "tocommitinmaster.txt").expect("Error al agregar el archivo");

        let test_commit1 = Commit::new(
            "prueba".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );
        git_commit(directory, test_commit1).expect("Error al commitear");

        // para copiar el commit a una branch remota
        git_branch_create(directory, "new_branch").expect("Error al crear la rama");

        // agrego un commit mas solo a master
        git_add(directory, "tocommitonlyonmaster1.txt").expect("Error al agregar el archivo");
        git_add(directory, "tocommitonlyonmaster2.txt").expect("Error al agregar el archivo");

        let test_commit2 = Commit::new(
            "para master".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );

        git_commit(directory, test_commit2).expect("Error al commitear");

        let new_branch_path = format!("{}/.git/refs/heads/{}", directory, "new_branch");
        let new_branch_file = open_file(&new_branch_path).expect("Error al abrir el archivo");
        let new_branch_hash = read_file_string(new_branch_file).expect("Error al leer el archivo");

        create_dir_all(format!("{}/.git/logs", directory)).expect("Error al crear el directorio");
        create_dir_all(format!("{}/.git/logs/refs", directory)).expect("Error al crear el directorio");
        create_dir_all(format!("{}/.git/logs/refs/remotes", directory)).expect("Error al crear el directorio");
        create_dir_all(format!("{}/.git/logs/refs/remotes/origin", directory)).expect("Error al crear el directorio");

        let log_new = open_file(format!("{}/.git/logs/refs/heads/new_branch", directory).as_str()).expect("Error al abrir el archivo");
        let log_new_content = read_file_string(log_new).expect("Error al leer el archivo");

        let log_remote = format!("{}/.git/logs/refs/remotes/origin/fetch", directory);
        create_file_replace(&log_remote, &log_new_content).expect("Error al crear el archivo");

        let remote_branch_path = format!("{}/.git/refs/remotes/origin/fetch", directory);
        create_file_replace(&remote_branch_path, &new_branch_hash).expect("Error al crear el archivo");

        let client: Client = Client::new(
            "Valen".to_string(), 
            "vlanzillotta@fi.uba.ar".to_string(),
            "19992020".to_string(),
            "9090".to_string(),
            "localhost".to_string(),
            "./".to_string(),
            "master".to_string(),
        );
        
        git_checkout_switch(directory, "new_branch").expect("Error al cambiar de rama");

        let result = git_merge(directory, "refs/remotes/origin/fetch", "master", client);

        fs::remove_dir_all(directory).expect("Falló al remover el directorio temporal");
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_last_commit_in_log() {
        // 
        // 455beb4810931b8b0b688103a152e318890dbd6e
        // parent 0000000000000000000000000000000000000000
        // author Valen <vlanzillotta@fi.uba.ar> 1701544637 -0300
        // committer Valen <vlanzillotta@fi.uba.ar> 1701544637 -0300

        // prueba
        // a0cb2895e88c84ffba96b94f6b1409464ad914dc
        // parent 455beb4810931b8b0b688103a152e318890dbd6e
        // author Valen <vlanzillotta@fi.uba.ar> 1701544637 -0300
        // committer Valen <vlanzillotta@fi.uba.ar> 1701544637 -0300

        // prueba otra
        // b378b7d03bce553e792408edc07c46296bac733a
        // parent a0cb2895e88c84ffba96b94f6b1409464ad914dc
        // author Valen <vlanzillotta@fi.uba.ar> 1701544638 -0300
        // committer Valen <vlanzillotta@fi.uba.ar> 1701544638 -0300

        // aa
        // 93af11079538db6ec316f552fdf9c969fd8eb582
        // parent b378b7d03bce553e792408edc07c46296bac733a
        // author Valen <vlanzillotta@fi.uba.ar> 1701544638 -0300
        // committer Valen <vlanzillotta@fi.uba.ar> 1701544638 -0300

        // bb
        let log = r#"
455beb4810931b8b0b688103a152e318890dbd6e
parent 0000000000000000000000000000000000000000
author Valen <vlanzillotta@fi.uba.ar> 1701544637 -0300
committer Valen <vlanzillotta@fi.uba.ar> 1701544637 -0300

prueba
a0cb2895e88c84ffba96b94f6b1409464ad914dc
parent 455beb4810931b8b0b688103a152e318890dbd6e
author Valen <vlanzillotta@fi.uba.ar> 1701544637 -0300
committer Valen <vlanzillotta@fi.uba.ar> 1701544637 -0300

prueba otra
b378b7d03bce553e792408edc07c46296bac733a
parent a0cb2895e88c84ffba96b94f6b1409464ad914dc
author Valen <vlanzillotta@fi.uba.ar> 1701544638 -0300
committer Valen <vlanzillotta@fi.uba.ar> 1701544638 -0300

aa
93af11079538db6ec316f552fdf9c969fd8eb582
parent b378b7d03bce553e792408edc07c46296bac733a
author Valen <vlanzillotta@fi.uba.ar> 1701544638 -0300
committer Valen <vlanzillotta@fi.uba.ar> 1701544638 -0300

bb"#;
        
        let last_commit = get_last_commit_in_log(log.to_string());
        let last_commit_formatted = r#"93af11079538db6ec316f552fdf9c969fd8eb582
parent b378b7d03bce553e792408edc07c46296bac733a
author Valen <vlanzillotta@fi.uba.ar> 1701544638 -0300
committer Valen <vlanzillotta@fi.uba.ar> 1701544638 -0300

bb"#;
        assert_eq!(last_commit, last_commit_formatted);
    }
}