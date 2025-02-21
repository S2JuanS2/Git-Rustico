use super::add::add_to_index;
use super::branch::{get_branch_current_hash, get_current_branch};
use super::cat_file::git_cat_file;
use super::checkout::extract_parent_hash;
use super::commit::{get_commits, merge_commit, Commit};
use super::errors::CommandsError;
use crate::commands::checkout::get_tree_hash;
use crate::commands::rm::remove_from_index;
use crate::consts::{DIRECTORY, FILE, GIT_DIR, PARENT_INITIAL, REFS_HEADS};
use crate::models::client::Client;
use crate::util::files::{create_file_replace, open_file, read_file_string};
use std::collections::HashMap;
use std::hash::Hash;
use std::io::{self, BufRead};

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
struct FileEntry {
    path: String,
    hash: String,
}

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
/// 'merge_branch': nombre de la rama a mergear
pub fn git_merge(
    directory: &str,
    current_branch: &str,
    merge_branch: &str,
    client: Client,
) -> Result<String, CommandsError> {
    let (result_merge, strategy) = perform_merge(current_branch, merge_branch, directory, "merge")?;

    if result_merge.contains("up to date") {
        return Ok(result_merge);
    }

    let path_current_branch = get_refs_path(directory, current_branch);
    let path_branch_to_merge = get_refs_path(directory, merge_branch);

    let current_branch_hash = get_branch_hash(&path_current_branch)?;
    let branch_to_merge_hash = get_branch_hash(&path_branch_to_merge)?;

    if !result_merge.contains("CONFLICT") {
        update_logs_refs(
            directory,
            strategy.clone(),
            current_branch,
            merge_branch,
            &current_branch_hash,
            &branch_to_merge_hash,
        )?;
        update_refs(
            directory,
            strategy,
            current_branch,
            merge_branch,
            &current_branch_hash,
            &branch_to_merge_hash,
            client.clone(),
        )?;
    }

    Ok(result_merge)
}

/// Chequea que estrategia se debe utilizar para el merge y procede a realizarlo.
/// ###Parametros:
/// 'current_branch': nombre de la rama actual
/// 'merge_branch': nombre de la rama a mergear
/// 'directory': directorio del repositorio local
/// 'merge_type': tipo de merge a realizar
pub fn perform_merge(
    current_branch: &str,
    merge_branch: &str,
    directory: &str,
    merge_type: &str,
) -> Result<(String, String), CommandsError> {
    if is_same_branch(current_branch, merge_branch) {
        return Err(CommandsError::IsSameBranch);
    }
    let is_head = current_branch == get_current_branch(directory)?;
    let mut result_merge = String::new();
    let common_ancestor = find_commit_common_ancestor(directory, current_branch, merge_branch)?;
    if is_up_to_date(directory, current_branch, merge_branch, &common_ancestor)? {
        return Ok(("Already up to date.".to_string(), "".to_string()));
    }
    let path_current_branch = get_refs_path(directory, current_branch);
    let path_branch_to_merge = get_refs_path(directory, merge_branch);
    let current_branch_hash = get_branch_hash(&path_current_branch)?;
    let branch_to_merge_hash = get_branch_hash(&path_branch_to_merge)?;

    let strategy = get_merge_strategy(common_ancestor, current_branch_hash.clone())?;
    if strategy == "Fast Forward" {
        let merge_tree = fast_forward(directory, merge_branch)?;
        if is_head {
            for file in merge_tree.iter() {
                let content_file = git_cat_file(directory, &file.hash, "-p")?;
                let full_path = format!("{}/{}", directory, file.path);
                create_file_replace(&full_path, &content_file)?;
                add_to_index(
                    format!("{}/{}", directory, GIT_DIR),
                    &file.path,
                    file.hash.clone(),
                )?;
            }
        }
        get_result_fast_forward(
            &mut result_merge,
            current_branch_hash.clone(),
            branch_to_merge_hash.clone(),
        );
    } else {
        let merge_tree = three_way_merge(directory, current_branch, merge_branch, merge_type)?;

        if merge_type == "pr" {
            if let Some((path, _)) = merge_tree.iter().find(|(_, status)| *status == "CONFLICT") {
                get_result_conflict(&mut result_merge, path);
                return Ok((result_merge, strategy));
            }
        }

        for (file, status) in merge_tree.iter() {
            if status == "CONFLICT" {
                get_result_conflict(&mut result_merge, file);
                return Ok((result_merge, strategy));
            } else if is_head {
                let content_file = git_cat_file(directory, &file.hash, "-p")?;
                let full_path = format!("{}/{}", directory, file.path);
                create_file_replace(&full_path, &content_file)?;
                add_to_index(
                    format!("{}/{}", directory, GIT_DIR),
                    &file.path,
                    file.hash.clone(),
                )?;
            }
        }
        result_merge.push_str("Merge made by the 'recursive' strategy.");
    }

    if is_head {
        update_work_directory(directory, &branch_to_merge_hash, &mut result_merge)?;
    }

    Ok((result_merge, strategy))
}

/// Esta función realiza un merge de una PR.
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'base_branch': nombre de la rama base
/// 'head_branch': nombre de la rama a mergear
/// 'owner': nombre del dueño de la PR
/// 'title': título de la PR
/// 'pr_number': número de la PR
/// 'repo_name': nombre del repositorio
pub fn merge_pr(
    directory: &str,
    base_branch: &str,
    head_branch: &str,
    owner: &str,
    title: &str,
    pr_number: &str,
    repo_name: &str,
) -> Result<String, CommandsError> {
    let (result_merge, strategy) = perform_merge(base_branch, head_branch, directory, "pr")?;
    let current_branch_commit = get_branch_current_hash(directory, base_branch.to_string())?;
    let merge_branch_commit = get_branch_current_hash(directory, head_branch.to_string())?;
    let mut result_merge_pr = String::new();

    if result_merge.contains("CONFLICT") {
        let first_line = result_merge.lines().next().unwrap();
        let conflict_path = first_line.strip_prefix("Auto-merging ").unwrap();
        result_merge_pr.push_str("Can’t automatically merge.\n");
        result_merge_pr.push_str(format!("Conflict in file:{}\n", conflict_path).as_str());
    } else {
        result_merge_pr.push_str("Automatic Merge was successfull\n");
        update_logs_refs(
            directory,
            strategy.clone(),
            base_branch,
            head_branch,
            &current_branch_commit,
            &merge_branch_commit,
        )?;
        update_refs_pr(
            directory,
            base_branch,
            head_branch,
            owner,
            title,
            pr_number,
            repo_name,
        )?;
    }

    Ok(result_merge)
}

/// Actualiza el repositorio en caso de recibir un commit con archivos eliminados
///
/// #Parametros:
/// 'directory': path del repositorio
/// 'branch_to_merge': nombre de la branch a mergear
fn update_work_directory(
    directory: &str,
    branch_to_merge_hash: &str,
    result_merge: &mut str,
) -> Result<(), CommandsError> {
    let content_commit = git_cat_file(directory, branch_to_merge_hash, "-p")?;
    let tree_hash = get_tree_hash(&content_commit).unwrap_or(PARENT_INITIAL);

    let parent_hash = extract_parent_hash(&content_commit).unwrap_or(PARENT_INITIAL);
    let parent_content = git_cat_file(directory, parent_hash, "-p")?;
    let parent_tree_hash = get_tree_hash(&parent_content).unwrap_or(PARENT_INITIAL);

    let mut vec_objects_parent_hash: Vec<String> = Vec::new();
    save_hash_objects(
        directory,
        &mut vec_objects_parent_hash,
        parent_tree_hash.to_string(),
    )?;

    let mut vec_objects_hash: Vec<String> = Vec::new();
    save_hash_objects(directory, &mut vec_objects_hash, tree_hash.to_string())?;
    let index_path = format!("{}/.git/index", directory);
    let index_file = open_file(index_path.as_str())?;
    let reader_index = io::BufReader::new(index_file);

    for line in reader_index.lines().map_while(Result::ok) {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() == 3 {
            let path = parts[0];
            let hash = parts[2];
            if vec_objects_hash.contains(&hash.to_string()) {
                // println!("Persiste");
            } else {
                let lines_result_conflict: Vec<&str> = result_merge.lines().collect();
                if lines_result_conflict.len() >= 4 {
                    let fourth_line = lines_result_conflict[3];
                    let mut chars = fourth_line.char_indices().filter(|&(_, c)| c == '/');
                    if let (Some(_first_pos), Some(second_pos)) = (chars.next(), chars.next()) {
                        let result = &fourth_line[(second_pos.0 + '/'.len_utf8())..];
                        if path == result {
                            // println!("Persiste, conflicto");
                        } else {
                            // println!("No persiste");
                            remove_from_index(directory, path, hash)?;
                        }
                    }
                } else if !vec_objects_parent_hash.contains(&hash.to_string()) {
                    // println!("Persiste");
                } else {
                    // println!("No persiste");
                    remove_from_index(directory, path, hash)?;
                }
            }
        }
    }
    Ok(())
}

/// Guarda en un vector recibido por parámetro los tree y blobs de un árbol principal
///
/// #Parametros:
/// 'directory': Path del repositorio
/// 'vec': Vector donde se guardaran los objetos
/// 'tree_hash': arbol principal donde se leeran los objetos.
fn save_hash_objects(
    directory: &str,
    vec: &mut Vec<String>,
    tree_hash: String,
) -> Result<(), CommandsError> {
    let tree = git_cat_file(directory, &tree_hash, "-p")?;
    for line in tree.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let file_mode = if parts[0] == FILE || parts[0] == DIRECTORY {
            parts[0]
        } else {
            parts[1]
        };
        let hash = parts[2];
        if file_mode == FILE {
            vec.push(hash.to_string());
        } else if file_mode == DIRECTORY {
            vec.push(hash.to_string());
            save_hash_objects(directory, vec, hash.to_string())?;
        }
    }
    Ok(())
}

/// Obtiene el resultado en caso de que haya un conflicto.
/// ###Parametros:
/// 'result_merge': resultado del merge
/// 'file': archivo que tiene conflicto
fn get_result_conflict(result_merge: &mut String, file: &FileEntry) {
    result_merge.push_str(format!("Auto-merging {}\n", file.path).as_str());
    result_merge
        .push_str(format!("CONFLICT (content): Merge conflict in {}\n", file.path).as_str());
    result_merge.push_str("Automatic merge failed; fix conflicts and then commit the result.\n");
    result_merge.push_str(format!("Conflict in file:{}\n", file.path).as_str());
}

/// Obtiene el resultado en caso de que haya un fast forward.
/// ###Parametros:
/// 'result_merge': resultado del merge
/// 'current_commit': commit actual
/// 'merge_commit': commit a mergear
fn get_result_fast_forward(
    result_merge: &mut String,
    current_commit: String,
    merge_commit: String,
) {
    result_merge.push_str(format!("Updating {}..{}\n", current_commit, merge_commit).as_str());
    result_merge.push_str("Fast-forward\n");
}

/// Obtiene la estrategia de merge a utilizar.
/// ###Parametros:
/// 'common_ancestor': ancestro común de las ramas a mergear
/// 'current_commit': commit actual
pub fn get_merge_strategy(
    common_ancestor: String,
    current_commit: String,
) -> Result<String, CommandsError> {
    if common_ancestor == current_commit {
        return Ok("Fast Forward".to_string());
    }
    Ok("Three Way".to_string())
}

// Función para verificar si dos branches son la misma
/// ###Parametros:
/// 'current_branch': nombre de la rama actual
/// 'merge_branch': nombre de la rama a mergear
fn is_same_branch(current_branch: &str, merge_branch: &str) -> bool {
    current_branch == merge_branch
}

// Función para actualizar los logs de las ramas
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'strategy': estrategia de merge
/// 'current_branch': nombre de la rama actual
/// 'merge_branch': nombre de la rama a mergear
/// 'current_commit': commit actual
/// 'merge_commit': commit a mergear
fn update_logs_refs(
    directory: &str,
    strategy: String,
    current_branch: &str,
    merge_branch: &str,
    current_commit: &str,
    merge_commit: &str,
) -> Result<(), CommandsError> {
    let log_merge_path = get_log_path(directory, merge_branch);
    let log_merge_file = open_file(&log_merge_path)?;
    let log_merge_content = read_file_string(log_merge_file)?;
    let log_current_path = get_log_path(directory, current_branch);
    let log_current_file = open_file(&log_current_path)?;
    let mut log_current_content = read_file_string(log_current_file)?;

    if strategy == "Fast Forward" {
        create_file_replace(&log_current_path, &log_merge_content)?;
    } else {
        let logs_current_branch = get_log_from_branch(directory, current_commit)?;
        let logs_merge_branch = get_log_from_branch(directory, merge_commit)?;
        let logs_just_in_merge_branch =
            logs_just_in_one_branch(logs_merge_branch.to_vec(), logs_current_branch.to_vec());
        // revertir el orden de logs_just_in_merge_branch
        let logs_just_in_merge_branch = logs_just_in_merge_branch.iter().rev().collect::<Vec<_>>();

        let new_commits: String = log_merge_content
            .lines()
            .skip_while(|&line| !line.contains(logs_just_in_merge_branch[0]))
            .collect::<Vec<&str>>()
            .join("\n");

        log_current_content.push_str(format!("\n{}", new_commits).as_str());
        create_file_replace(&log_current_path, &log_current_content)?;
    }
    Ok(())
}

// Función para actualizar las referencias de las ramas
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'strategy': estrategia de merge
/// 'current_branch': nombre de la rama actual
/// 'merge_branch': nombre de la rama a mergear
/// 'current_branch_commit': commit actual
/// 'merge_branch_commit': commit a mergear
/// 'client': cliente que realizó el merge
fn update_refs(
    directory: &str,
    strategy: String,
    current_branch: &str,
    merge_branch: &str,
    current_branch_commit: &str,
    merge_branch_commit: &str,
    client: Client,
) -> Result<(), CommandsError> {
    let current_commit_path = format!(
        "{}/{}/{}/{}",
        directory, GIT_DIR, REFS_HEADS, current_branch
    );
    let mut merge_commit_path =
        format!("{}/{}/{}/{}", directory, GIT_DIR, REFS_HEADS, merge_branch);
    if merge_branch.contains('/') {
        merge_commit_path = format!("{}/{}/{}", directory, GIT_DIR, merge_branch);
    }
    let merge_commit_file = open_file(&merge_commit_path)?;
    let merge_commit_content = read_file_string(merge_commit_file)?;

    if strategy == "Fast Forward" {
        create_file_replace(&current_commit_path, &merge_commit_content)?;
    } else {
        let commit = Commit::new(
            "Merge Commit".to_string(),
            client.get_name().to_string(),
            client.get_email().to_string(),
            client.get_name().to_string(),
            client.get_email().to_string(),
        );
        merge_commit(
            directory,
            commit,
            current_branch_commit,
            merge_branch_commit,
        )?;
    }
    Ok(())
}

// Función para actualizar las referencias de las ramas en caso de una PR. Se commitea un merge pull request.
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'base_branch': nombre de la rama base
/// 'head_branch': nombre de la rama a mergear
/// 'owner': nombre del dueño de la PR
/// 'title': título de la PR
/// 'pr_number': número de la PR
/// 'repo_name': nombre del repositorio
fn update_refs_pr(
    directory: &str,
    base_branch: &str,
    head_branch: &str,
    owner: &str,
    title: &str,
    pr_number: &str,
    repo_name: &str,
) -> Result<(), CommandsError> {
    let current_branch_commit = get_branch_current_hash(directory, base_branch.to_string())?;
    let branch_to_merge_commit = get_branch_current_hash(directory, head_branch.to_string())?;

    let message = format!(
        "Merge pull request #{} from {}/{}. Title {}",
        pr_number, repo_name, head_branch, title
    );
    let owner_email = format!("{}@users.noreply.rusteam.com", owner);
    let commiter_name = "Rusteam".to_string();
    let commiter_email = "noreply@rusteam.com".to_string();
    let commit = Commit::new(
        message,
        owner.to_string(),
        owner_email,
        commiter_name,
        commiter_email,
    );
    merge_commit(
        directory,
        commit,
        &current_branch_commit,
        &branch_to_merge_commit,
    )?;
    Ok(())
}

// Función para encontrar el ancestro común de dos branches, es decir, el commit más reciente que comparten ambas ramas.
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'current_branch': nombre de la rama actual
/// 'branch_to_merge': nombre de la rama a mergear
pub fn find_commit_common_ancestor(
    directory: &str,
    current_branch: &str,
    branch_to_merge: &str,
) -> Result<String, CommandsError> {
    let mut commit_common_ancestor = String::new();
    let commits_current = get_commits(directory, current_branch)?;
    let commits_merge = get_commits(directory, branch_to_merge)?;
    for commit_current in commits_current.iter().rev() {
        for commit_merge in commits_merge.iter().rev() {
            if commit_current == commit_merge {
                commit_common_ancestor = commit_current.to_string();
            }
        }
    }
    Ok(commit_common_ancestor)
}

// Función para verificar si la current_branch ya esta actualizada con la merge_branch. Si las ramas tienen el mismo hash, o si el ancestro común es igual
/// al hash de la rama a mergear, entonces la rama ya esta actualizada.
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'current_branch': nombre de la rama actual
/// 'merge_branch': nombre de la rama a mergear
/// 'common_ancestor': ancestro común de las ramas a mergear
fn is_up_to_date(
    directory: &str,
    current_branch: &str,
    merge_branch: &str,
    common_ancestor: &str,
) -> Result<bool, CommandsError> {
    let path_current_branch = get_refs_path(directory, current_branch);
    let path_branch_to_merge = get_refs_path(directory, merge_branch);

    let current_branch_hash = get_branch_hash(&path_current_branch)?;
    let branch_to_merge_hash = get_branch_hash(&path_branch_to_merge)?;
    if current_branch_hash == branch_to_merge_hash || common_ancestor == branch_to_merge_hash {
        return Ok(true);
    }
    Ok(false)
}

// Función para realizar un Fast Forward
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'merge_branch': nombre de la rama a mergear
fn fast_forward(directory: &str, merge_branch: &str) -> Result<Vec<FileEntry>, CommandsError> {
    let path_branch_to_merge = get_refs_path(directory, merge_branch);
    let branch_to_merge_hash = get_branch_hash(&path_branch_to_merge)?;
    let merge_commit_content = git_cat_file(directory, &branch_to_merge_hash, "-p")?;

    // Obtener el tree del merge_branch
    let merge_tree_hash =
        get_tree_hash(&merge_commit_content).ok_or(CommandsError::InvalidTreeHashError)?;
    let mut files_in_tree = Vec::new();
    get_files_in_tree(
        directory,
        merge_tree_hash,
        &mut "".to_string(),
        &mut files_in_tree,
    )?;

    Ok(files_in_tree)
}

// Función para realizar un Three way merge
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'current_branch': nombre de la rama actual
/// 'merge_branch': nombre de la rama a mergear
/// 'merge_type': tipo de merge a realizar
fn three_way_merge(
    directory: &str,
    current_branch: &str,
    merge_branch: &str,
    merge_type: &str,
) -> Result<HashMap<FileEntry, String>, CommandsError> {
    let path_current_branch = get_refs_path(directory, current_branch);
    let path_branch_to_merge = get_refs_path(directory, merge_branch);

    let current_branch_hash = get_branch_hash(&path_current_branch)?;
    let branch_to_merge_hash = get_branch_hash(&path_branch_to_merge)?;

    let current_commit_content = git_cat_file(directory, &current_branch_hash, "-p")?;
    let merge_commit_content = git_cat_file(directory, &branch_to_merge_hash, "-p")?;

    let current_tree_hash =
        get_tree_hash(&current_commit_content).ok_or(CommandsError::InvalidTreeHashError)?;
    let merge_tree_hash =
        get_tree_hash(&merge_commit_content).ok_or(CommandsError::InvalidTreeHashError)?;

    // Obtener los files de ambas branches con el formato FileEntry (path, hash)
    let mut files_in_current_tree = Vec::new();
    let mut files_in_merge_tree = Vec::new();
    get_files_in_tree(
        directory,
        current_tree_hash,
        &mut "".to_string(),
        &mut files_in_current_tree,
    )?;
    get_files_in_tree(
        directory,
        merge_tree_hash,
        &mut "".to_string(),
        &mut files_in_merge_tree,
    )?;

    // Voy a devolver una estructura que sea un HashMap<FileEntry, String> con el FileEntry de los archivos y sus blobs y un string con OK o CONFLICT
    let mut result: HashMap<FileEntry, String> = HashMap::new();

    for file in files_in_merge_tree.iter() {
        if let Some(current_file) = files_in_current_tree.iter().find(|f| f.path == file.path) {
            if current_file.hash != file.hash {
                // El archivo existe en current_branch pero fue modificado en merge_branch
                result.insert(file.clone(), "CONFLICT".to_string());
                if merge_type == "merge" || merge_type == "rebase" {
                    check_each_line(directory, current_file, file, merge_branch)?;
                }
            }
        } else {
            // El archivo no existe en current_branch, es un archivo nuevo
            result.insert(file.clone(), "OK".to_string());
        }
    }

    Ok(result)
}

/// Obtiene los archivos de un tree.
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'tree_hash': hash del tree
/// 'path': path del archivo
fn get_files_in_tree(
    directory: &str,
    tree_hash: &str,
    path: &mut str,
    files_in_tree: &mut Vec<FileEntry>,
) -> Result<(), CommandsError> {
    let tree_content = git_cat_file(directory, tree_hash, "-p")?;
    for line in tree_content.lines() {
        if !line.is_empty() {
            let tree_parts: Vec<&str> = line.split_whitespace().collect();
            let mut current_path = if path.is_empty() {
                tree_parts[1].to_string()
            } else {
                format!("{}/{}", path, tree_parts[1])
            };

            if git_cat_file(directory, tree_parts[2], "-t")? == "tree" {
                get_files_in_tree(directory, tree_parts[2], &mut current_path, files_in_tree)?;
            } else {
                files_in_tree.push(FileEntry {
                    path: current_path,
                    hash: tree_parts[2].to_string(),
                });
            }
        }
    }
    Ok(())
}

/// Chequea cada linea de los archivos que difieren entre las ramas a mergear. Esto solo se hace en caso de merge o rebase, NO en caso de un merge PR.
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'current_file': archivo de la rama actual
/// 'merge_file': archivo de la rama a mergear
/// 'merge_branch': nombre de la rama a mergear
fn check_each_line(
    directory: &str,
    current_file: &FileEntry,
    merge_file: &FileEntry,
    merge_branch: &str,
) -> Result<(), CommandsError> {
    let current_file_content = git_cat_file(directory, &current_file.hash, "-p")?;
    let merge_file_content = git_cat_file(directory, &merge_file.hash, "-p")?;

    let mut current_file_lines = current_file_content.lines();
    let mut merge_file_lines = merge_file_content.lines();
    let mut line_current = current_file_lines.next();
    let mut line_merge = merge_file_lines.next();

    let mut new_content_file = String::new();

    while line_current.is_some() && line_merge.is_some() {
        if line_current != line_merge {
            new_content_file.push_str("<<<<<<< HEAD\n");
            if let Some(line_current) = line_current {
                new_content_file.push_str(line_current);
            }
            new_content_file.push_str("\n=======\n");
            if let Some(line_merge) = line_merge {
                new_content_file.push_str(line_merge);
            }
            new_content_file.push_str("\n>>>>>>> ");
            new_content_file.push_str(merge_branch);
            new_content_file.push('\n');
        } else {
            if let Some(line_current) = line_current {
                new_content_file.push_str(line_current);
            }
            new_content_file.push('\n');
        }
        line_current = current_file_lines.next();
        line_merge = merge_file_lines.next();
    }
    let full_path = format!("{}/{}", directory, current_file.path);
    create_file_replace(&full_path, &new_content_file)?;
    Ok(())
}

/// Obtiene el log de la rama pasada por parametro.
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'hash': hash de la rama
pub fn get_log_from_branch(directory: &str, hash: &str) -> Result<Vec<String>, CommandsError> {
    let mut log_current_branch: Vec<String> = Vec::new();
    let commit_content = git_cat_file(directory, hash, "-p")?;
    add_commit_to_log(directory, &mut log_current_branch, hash, commit_content)?;

    Ok(log_current_branch)
}

fn add_commit_to_log(
    directory: &str,
    log_current_branch: &mut Vec<String>,
    hash: &str,
    commit_content: String,
) -> Result<(), CommandsError> {
    log_current_branch.push(hash.to_string());
    if let Some(parent_hash) = extract_parent_hash(&commit_content) {
        let commit_content = git_cat_file(directory, parent_hash, "-p")?;
        add_commit_to_log(directory, log_current_branch, parent_hash, commit_content)?;
    }
    Ok(())
}

/// Obtiene los logs que difieren entre las ramas a mergear.
/// ###Parametros:
/// 'log_current_branch': logs de la branch actual
/// 'log_other_branch': logs de otra branch
pub fn logs_just_in_one_branch(
    log_current_branch: Vec<String>,
    log_other_branch: Vec<String>,
) -> Vec<String> {
    let logs_just_in_current_branch = log_current_branch
        .iter()
        .filter(|commit| !log_other_branch.contains(commit))
        .collect::<Vec<_>>();
    logs_just_in_current_branch
        .iter()
        .map(|commit| commit.to_string())
        .collect::<Vec<_>>()
}

/// Obtiene el path del archivo de una rama (en refs/heads si es local o en refs/remotes si es remota).
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'branch_name': nombre de la rama
pub fn get_refs_path(directory: &str, branch_name: &str) -> String {
    let mut path_branch_to_merge = format!("{}/.git/refs/heads/{}", directory, branch_name);
    if branch_name.contains("remotes") {
        path_branch_to_merge = format!("{}/.git/{}", directory, branch_name);
    } else if branch_name.contains('/') && !branch_name.contains("remotes") {
        path_branch_to_merge = format!("{}/.git/refs/remotes/{}", directory, branch_name);
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
            log_path = format!(
                "{}/.git/logs/refs/remotes/{}/{}",
                directory, branch_path[2], branch_path[3]
            );
        } else {
            log_path = format!("{}/.git/logs/refs/remotes/{}", directory, branch_path[2]);
        }
    } else {
        let branch_path = branch_name.split('/').collect::<Vec<_>>();
        if branch_path.len() == 2 {
            log_path = format!(
                "{}/.git/logs/refs/remotes/{}/{}",
                directory, branch_path[0], branch_path[1]
            );
        }
    }
    log_path
}

/// Obtiene el hash de una rama.
/// ###Parametros:
/// 'path_current_branch': path del archivo de la rama actual
/// 'path_branch_to_merge': path del archivo de la rama a mergear
pub fn get_branch_hash(path_branch: &str) -> Result<String, CommandsError> {
    let branch_file = open_file(path_branch)?;
    let branch_hash = read_file_string(branch_file)?;
    Ok(branch_hash)
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
