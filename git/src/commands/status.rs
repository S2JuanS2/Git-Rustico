use crate::consts::*;
use super::check_ignore::{check_gitignore, get_gitignore_content};
use super::errors::CommandsError;
use crate::models::client::Client;
use crate::util::files::{open_file, read_file, read_file_string};
use crate::util::formats::hash_generate;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use super::cat_file::git_cat_file;

/// Esta función se encarga de llamar al comando status con los parametros necesarios.
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función status
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_status(args: Vec<&str>, client: Client) -> Result<String, CommandsError> {
    if !args.is_empty() {
        return Err(CommandsError::InvalidArgumentCountStatusError);
    }
    let directory = client.get_directory_path();
    git_status(directory)
}

/// Devuelve el nombre de la rama actual.
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
fn get_head_branch(directory: &str) -> Result<String, CommandsError> {
    // "directory/.git/HEAD"
    let directory_git = format!("{}/{}", directory, GIT_DIR);
    let head_file_path = Path::new(&directory_git).join(HEAD);

    let head_file = File::open(head_file_path);
    let mut head_file = match head_file {
        Ok(file) => file,
        Err(_) => return Err(CommandsError::OpenFileError),
    };
    let mut head_branch: String = String::new();
    let read_head_file = head_file.read_to_string(&mut head_branch);
    let _ = match read_head_file {
        Ok(file) => file,
        Err(_) => return Err(CommandsError::ReadFileError),
    };
    let head_branch_name = head_branch.split('/').last();
    let head_branch_name = match head_branch_name {
        Some(name) => name,
        None => return Err(CommandsError::HeadBranchError),
    };
    let head_branch_name = head_branch_name.trim().to_string();

    Ok(head_branch_name)
}

/// Compara los hashes de los archivos del directorio de trabajo con los del index e imprime el estado
/// del repositorio local, incluyendo las diferencias entre los archivos locales y los archivos que ya
/// fueron agregados al staging area.
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
pub fn git_status(directory: &str) -> Result<String, CommandsError> {
    let directory_git = format!("{}/{}", directory, GIT_DIR);

    let index_content = get_index_content(&directory_git)?;

    let index_files = get_lines_in_index(index_content);

    let working_directory_hash_list = get_hashes_working_directory(directory)?;
    let index_hashes = get_hashes_index(index_files)?;
    let (updated_files_list, untracked_files_list, staged_files_list, deleted_files_list) =
        compare_hash_lists(&working_directory_hash_list, &index_hashes, directory);
    let files_not_commited_list = check_for_commit(directory, staged_files_list)?;
    let value = print_changes(
        updated_files_list,
        untracked_files_list,
        files_not_commited_list,
        deleted_files_list,
        directory,
    )?;

    Ok(value)
}

pub fn get_lines_in_index(index_content: String) -> Vec<String> {
    let lines: Vec<String> = index_content.lines().map(String::from).collect();
    let mut index_files: Vec<String> = Vec::new();
    if !lines.is_empty() {
        for line in lines {
            index_files.push(line);
        }
    }
    index_files
}

/// Devuelve el contenido del archivo index.
/// ###Parámetros:
/// 'directory_git': directorio del repositorio local.
pub fn get_index_content(directory_git: &str) -> Result<String, CommandsError> {
    let index_path = format!("{}/index", directory_git);
    let index_file = File::open(index_path);
    let mut index_file = match index_file {
        Ok(file) => file,
        Err(_) => return Err(CommandsError::OpenFileError),
    };
    let mut index_content: String = String::new();
    let read_index_file = index_file.read_to_string(&mut index_content);
    let _ = match read_index_file {
        Ok(file) => file,
        Err(_) => return Err(CommandsError::ReadFileError),
    };
    Ok(index_content)
}

/// Imprime el resultado de git status.
/// ###Parámetros:
/// 'updated_files_list': vector con los archivos que se modificaron y no se actualizaron en el staging area.
/// 'untracked_files_list': vector con los archivos que no estan trackeados.
/// 'files_not_commited_list': vector con los archivos que estan en el staging area y se van a incluir en el proximo commit.
/// 'deleted_files_list': vector con los archivos que se eliminaron del working directory pero siguen en el index.
/// 'directory': directorio del repositorio local.
fn print_changes(
    updated_files_list: Vec<(String, String)>,
    untracked_files_list: Vec<(String, String)>,
    files_not_commited_list: Vec<String>,
    deleted_files_list: Vec<String>,
    directory: &str,
) -> Result<String, CommandsError> {
    let mut formatted_result = String::new();
    let head_branch_name = get_head_branch(directory)?;

    formatted_result.push_str("On branch ");
    formatted_result.push_str(&head_branch_name);
    if updated_files_list.is_empty()
        && untracked_files_list.is_empty()
        && files_not_commited_list.is_empty()
        && deleted_files_list.is_empty()
    {
        branch_up_to_date(&mut formatted_result, head_branch_name);
    }
    if !updated_files_list.is_empty() || !deleted_files_list.is_empty() {
        branch_with_untracked_changes(
            &mut formatted_result,
            &updated_files_list,
            &untracked_files_list,
            &deleted_files_list,
            directory,
        );
    }
    if !untracked_files_list.is_empty() {
        branch_with_untracked_files(
            &mut formatted_result,
            &untracked_files_list,
            &files_not_commited_list,
            directory,
        );
    }
    if !files_not_commited_list.is_empty() {
        branch_missing_commits(&mut formatted_result, &files_not_commited_list);
    }

    Ok(formatted_result)
}

/// Muestra los archivos con cambios que no estan en el staging area.
/// ###Parámetros:
/// 'formatted_result': string con el resultado del status formateado.
/// 'updated_files_list': vector con los nombres de los archivos que se modificaron y no se actualizaron en el staging area.
/// 'untracked_files_list': vector con los nombres de los archivos que no estan en el staging area.
/// 'deleted_files_list': vector con los nombres de los archivos que se eliminaron del working directory pero siguen en el index.
/// 'directory': directorio del repositorio local.
fn branch_with_untracked_changes(
    formatted_result: &mut String,
    updated_files_list: &Vec<(String, String)>,
    untracked_files_list: &Vec<(String, String)>,
    deleted_files_list: &Vec<String>,
    directory: &str,
) {
    formatted_result.push_str("\nChanges not staged for commit:\n");
    formatted_result.push_str("  (use \"git add <file>...\" to update what will be committed)\n");
    formatted_result.push_str(
        "  (use \"git checkout -- <file>...\" to discard changes in working directory)\n",
    );

    if !updated_files_list.is_empty() {
        for file in updated_files_list {
            let file_path = &file.0[directory.len() + 1..];
            formatted_result.push_str(&format!("\n\tmodified:\t\t{}\n", file_path));
        }
    }
    if !deleted_files_list.is_empty() {
        for file in deleted_files_list {
            formatted_result.push_str(&format!("\n\tdeleted:\t\t{}\n", file));
        }
    }
    if untracked_files_list.is_empty() {
        formatted_result
            .push_str("\nno changes added to commit (use \"git add\" and/or \"git commit -a\")\n");
    }
}

/// Muestra los archivos que no estan trackeados.
/// ###Parámetros:
/// 'formatted_result': string con el resultado del status formateado.
/// 'untracked_files_list': vector con los nombres de los archivos que no estan en el staging area.
/// 'files_not_commited_list': vector con los nombres de los archivos que estan en el staging area y se van a incluir en el proximo commit.
/// 'directory': directorio del repositorio local.
fn branch_with_untracked_files(
    formatted_result: &mut String,
    untracked_files_list: &Vec<(String, String)>,
    files_not_commited_list: &Vec<String>,
    directory: &str,
) {
    formatted_result.push_str("\nUntracked files:\n");
    formatted_result
        .push_str("  (use \"git add <file>...\" to include in what will be committed)\n");

    if !untracked_files_list.is_empty() {
        for file in untracked_files_list {
            let file_path = &file.0[directory.len() + 1..];
            formatted_result.push_str(&format!("\n\tnew file: \t{}", file_path));
        }
    }
    if files_not_commited_list.is_empty() {
        formatted_result.push_str(
            "\n\nnothing added to commit but untracked files present (use \"git add\" to track)\n",
        );
    }
}

/// Muestra los archivos que estan en el staging area y van a ser incluidos en el proximo commit.
/// ###Parámetros:
/// 'formatted_result': string con el resultado del status formateado.
/// 'files_not_commited_list': vector con los nombres de los archivos que estan en el staging area y se van a incluir en el proximo commit.
fn branch_missing_commits(formatted_result: &mut String, files_not_commited_list: &Vec<String>) {
    formatted_result.push_str("\n\nChanges to be committed:\n");
    formatted_result.push_str("  (use \"git reset HEAD <file>...\" to unstage)\n\n");

    for file in files_not_commited_list {
        formatted_result.push_str(&format!("\tmodified:\t{}\n", file));
    }
}

/// Muestra que el repositorio local esta actualizado.
/// ###Parámetros:
/// 'formatted_result': string con el resultado del status formateado.
/// 'head_branch_name': nombre de la rama actual.
fn branch_up_to_date(formatted_result: &mut String, head_branch_name: String) {
    formatted_result.push_str(&format!(
        "\nYour branch is up to date with 'origin/{}'.\n",
        head_branch_name
    ));
    formatted_result.push_str("\nnothing to commit, working tree clean\n");
}

/// Compara los hashes de los archivos del directorio de trabajo con los del index y devuelve cuatro vectores:
/// - updated_files_list: vector con los archivos que se modificaron y no se actualizaron en el staging area.
/// - untracked_files_list: vector con los archivos que no estan trackeados.
/// - staged_files_list: vector con los archivos que estan en el staging area y se van a incluir en el proximo commit.
/// - deleted_files_list: vector con los archivos que se eliminaron del working directory pero siguen en el index.
/// ###Parámetros:
/// 'working_directory_hash_list': HashMap con los nombres de los archivos en el working directory y sus hashes.
/// 'index_hashes': vector con los nombres de los archivos en el index y sus hashes.
/// 'directory': directorio del repositorio local.
pub fn compare_hash_lists(
    working_directory_hash_list: &HashMap<String, String>,
    index_hashes: &Vec<(String, String)>,
    directory: &str,
) -> (
    Vec<(String, String)>,
    Vec<(String, String)>,
    Vec<(String, String)>,
    Vec<String>,
) {
    let mut updated_files_list: Vec<(String, String)> = Vec::new();
    let mut untracked_files_list: Vec<(String, String)> = Vec::new();
    let mut staged_files_list: Vec<(String, String)> = Vec::new();
    for working_dir_hash in working_directory_hash_list {
        let mut found_hash_in_index = false;
        for index_hash in index_hashes {
            let file_path = &working_dir_hash.0[directory.len() + 1..];
            if file_path == index_hash.0 {
                // el archivo esta trackeado, debo ver si esta en su ultima version
                found_hash_in_index = true;
                if working_dir_hash.1 != &index_hash.1 {
                    updated_files_list.push((
                        working_dir_hash.0.to_string(),
                        working_dir_hash.1.to_string(),
                    ));
                } else {
                    staged_files_list.push((
                        working_dir_hash.0.to_string(),
                        working_dir_hash.1.to_string(),
                    ));
                }
            }
        }
        if !found_hash_in_index {
            untracked_files_list.push((
                working_dir_hash.0.to_string(),
                working_dir_hash.1.to_string(),
            ));
        }
    }
    let deleted_files_list =
        check_for_deleted_files(index_hashes, working_directory_hash_list, directory);
    (
        updated_files_list,
        untracked_files_list,
        staged_files_list,
        deleted_files_list,
    )
}

/// Devuelve un vector con los nombres de los archivos que se eliminaron del working directory pero siguen en el index.
/// ###Parámetros:
/// 'index_hashes': vector con los nombres de los archivos en el index y sus hashes.
/// 'working_directory_hash_list': HashMap con los nombres de los archivos en el working directory y sus hashes.
/// 'directory': directorio del repositorio local.
pub fn check_for_deleted_files(
    index_hashes: &Vec<(String, String)>,
    working_directory_hash_list: &HashMap<String, String>,
    directory: &str,
) -> Vec<String> {
    let index_files_len = &index_hashes.len();
    let working_directory_files_len = &working_directory_hash_list.len();
    let mut deleted_files_list: Vec<String> = Vec::new();
    if index_files_len > working_directory_files_len {
        for index_hash in index_hashes {
            let mut found_hash_in_index = false;
            for working_dir_hash in working_directory_hash_list {
                let file_path = &working_dir_hash.0[directory.len() + 1..];
                if file_path == index_hash.0 {
                    found_hash_in_index = true;
                }
            }
            if !found_hash_in_index {
                deleted_files_list.push(index_hash.0.to_string());
            }
        }
    }
    deleted_files_list
}

/// Se para en el ultimo commit de la branch actual y reconstruye el arbol de archivos incluidos
/// en ese commit para ver si los archivos que estan en el staging area fueron incluidos en ese commit.
/// Si no fueron incluidos, los agrega a un vector de 'files_not_commited_list' (que devuelve).
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
/// 'staged_files_list': vector con los nombres de los archivos en el staging area y sus hashes
fn check_for_commit(
    directory: &str,
    staged_files_list: Vec<(String, String)>,
) -> Result<Vec<String>, CommandsError> {
    let mut files_not_commited_list: Vec<String> = Vec::new();
    if !staged_files_list.is_empty() {
        let head_branch = get_head_branch(directory)?;
        let head_branch = format!("{}/.git/refs/heads/{}", directory, head_branch);
        if fs::metadata(&head_branch).is_err() {
            files_not_commited_list = staged_files_list
                .iter()
                .map(|file| file.0.to_string())
                .collect();
            return Ok(files_not_commited_list);
        }
        let head_branch_file = open_file(&head_branch)?;
        let head_branch_commmit = read_file_string(head_branch_file)?;
        for file in staged_files_list {
            let commited = get_files_in_commit(directory, &head_branch_commmit, &file.1)?;
            if !commited {
                files_not_commited_list.push(file.0.to_string());
            }
        }
    }
    Ok(files_not_commited_list)
}

/// Devuelve un vector con los nombres de los archivos en el index y sus hashes.
/// ###Parámetros:
/// 'index_files_list': vector con las lineas del index.
pub fn get_hashes_index(index_files_list: Vec<String>) -> Result<Vec<(String, String)>, CommandsError> {
    let mut index_hashes: Vec<(String, String)> = Vec::new();
    for file in index_files_list {
        let parts: Vec<&str> = file.split(' ').collect();
        let file_name = match parts.first() {
            Some(name) => name,
            None => return Err(CommandsError::GenericError),
        };
        let file_hash = match parts.last() {
            Some(hash) => hash,
            None => return Err(CommandsError::GenericError),
        };
        index_hashes.push((file_name.to_string(), file_hash.to_string()));
    }
    Ok(index_hashes)
}

/// Reconstruye el arbol de archivos incluidos en un commit y devuelve un booleano que indica si el archivo
/// que se le pasa como parametro fue incluido en ese commit.
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
/// 'commit_actual': hash del commit actual.
/// 'file_hash': hash del archivo que se quiere buscar.
fn get_files_in_commit(
    directory: &str,
    commit_actual: &str,
    file_hash: &str,
) -> Result<bool, CommandsError> {
    let mut commited = false;
    if !commit_actual.is_empty() {
        let commit_content = git_cat_file(directory, commit_actual, "-p")?;
        let commit_lines = commit_content.split('\n');
        let mut parent_commit = "";
        for line in commit_lines {
            if line.starts_with("parent") {
                if let Some(parent_hash) = line.split(' ').last() {
                    parent_commit = parent_hash;
                }
            }
            if line.starts_with("tree") {
                if let Some(tree_hash) = line.split(' ').last() {
                    get_tree_content(
                        directory,
                        tree_hash,
                        file_hash,
                        &mut commited,
                        parent_commit,
                    )?;
                }
            }
        }
    }
    Ok(commited)
}

/// Recorre el arbol de archivos que se le pasa como parametro y busca en ellos el hash del archivo que
/// se le pasa como parametro.
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
/// 'tree_hash': hash del arbol de archivos.
/// 'file_hash': hash del archivo que se quiere buscar.
/// 'commited': booleano que indica si el archivo que se quiere buscar fue incluido en un commit.
/// 'parent_commit': hash del commit padre.
fn get_tree_content(
    directory: &str,
    tree_hash: &str,
    file_hash: &str,
    commited: &mut bool,
    parent_commit: &str,
) -> Result<(), CommandsError> {
    let tree_content = git_cat_file(directory, tree_hash, "-p")?;
    let tree_lines = tree_content.split('\n');
    for tree_line in tree_lines {
        if !tree_line.is_empty(){
            let tree_parts: Vec<&str> = tree_line.split_whitespace().collect();
            if git_cat_file(directory, tree_parts[2], "-t")? == "tree" {
                get_tree_content(directory, tree_parts[2], file_hash, commited, parent_commit)?;
            }
            if tree_parts[2] == file_hash {
                *commited = true;
                return Ok(());
            }
        }
    }
    if !*commited && parent_commit != "0000000000000000000000000000000000000000" {
        get_files_in_commit(directory, parent_commit, file_hash)?;
    }
    Ok(())
}

/// Devuelve un HashMap con los nombres de los archivos en el working directory y sus hashes correspondientes.
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
pub fn get_hashes_working_directory(directory: &str) -> Result<HashMap<String, String>, CommandsError> {
    let mut working_directory_hash_list: HashMap<String, String> = HashMap::new();
    let working_directory = directory.to_string();
    let gitignore_content = get_gitignore_content(directory)?;
    calculate_directory_hashes(&working_directory, &mut working_directory_hash_list, &gitignore_content)?;
    Ok(working_directory_hash_list)
}

/// Recorre el directorio de trabajo recursivamente y devuelve un HashMap con los nombres de los archivos y
/// sus hashes correspondientes.
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
/// 'hash_list': HashMap con los nombres de los archivos en el working directory y sus hashes.
pub fn calculate_directory_hashes(
    directory: &str,
    hash_list: &mut HashMap<String, String>,
    gitignore_content: &str,
) -> Result<(), CommandsError> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(_) => return Err(CommandsError::ReadDirError),
    };
    let mut ignored_files: Vec<String> = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => return Err(CommandsError::DirEntryError),
        };
        let path = entry.path();

        let file_name = entry.file_name();
        if let Some(file_name) = file_name.to_str() {
            if file_name.starts_with('.') {
                continue;
            }
            check_gitignore(file_name, &mut ignored_files, gitignore_content)?;
            if !ignored_files.is_empty() && ignored_files.contains(&file_name.to_string()) {
                continue;
            }
        }

        create_hash_working_dir(path, hash_list, gitignore_content)?;
    }
    Ok(())
}

/// Crea el hash de un archivo del working directory y lo agrega a un HashMap.
/// ###Parámetros:
/// 'path': path del archivo.
/// 'hash_list': HashMap con los nombres de los archivos en el working directory y sus hashes.
fn create_hash_working_dir(
    path: PathBuf,
    hash_list: &mut HashMap<String, String>,
    gitignore_content: &str,
) -> Result<(), CommandsError> {
    if path.is_dir() {
        if let Some(path_str) = path.to_str() {
            calculate_directory_hashes(path_str, hash_list, gitignore_content)?;
        }
    } else if let Some(file_name_str) = path.to_str() {
        let file = open_file(file_name_str)?;
        let content = read_file(file)?;

        let header = format!("{} {}\0", BLOB, content.len());
        let store = header + String::from_utf8_lossy(&content).as_ref();
        let hash_object = hash_generate(&store);

        hash_list.insert(file_name_str.to_string(), hash_object);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{commands::{
        add::git_add,
        commit::{git_commit, Commit},
        init::git_init,
    }, util::files::create_file_replace};

    use super::*;
    use std::io::Write;

    #[test]
    fn test_git_status() {
        let directory: &str = "./test_status";
        git_init(directory).expect("Error al ejecutar git init");

        let file_path = format!("{}/{}", directory, "testfile.rs");
        let mut file = fs::File::create(&file_path).expect("Falló al crear el archivo");
        file.write_all(b"Hola Mundo")
            .expect("Error al escribir en el archivo");

        let file_path2 = format!("{}/{}", directory, "main.rs");
        let mut file = fs::File::create(&file_path2).expect("Falló al crear el archivo");
        file.write_all(b"Chau Mundo")
            .expect("Error al escribir en el archivo");

        let result_before_add = git_status(directory);

        git_add(directory, "testfile.rs").expect("Error al ejecutar git add");

        let result_after_add = git_status(directory);
        let result_after = "On branch master\nUntracked files:\n  (use \"git add <file>...\" to include in what will be committed)\n\n\tnew file: \tmain.rs\n\nChanges to be committed:\n  (use \"git reset HEAD <file>...\" to unstage)\n\n\tmodified:\t./test_status\\testfile.rs\n";
        assert_eq!(result_after_add, Ok(result_after.to_string()));

        let test_commit1 = Commit::new(
            "prueba".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );
        git_commit(directory, test_commit1).expect("Error al commitear");

        let result_after_commit = git_status(directory);
        let result_after_commit_ = "On branch master\nUntracked files:\n  (use \"git add <file>...\" to include in what will be committed)\n\n\tnew file: \tmain.rs\n\nnothing added to commit but untracked files present (use \"git add\" to track)\n";
        assert_eq!(result_after_commit, Ok(result_after_commit_.to_string()));

        git_add(directory, "main.rs").expect("Error al ejecutar git add");

        let test_commit2 = Commit::new(
            "prueba2".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );
        git_commit(directory, test_commit2).expect("Error al commitear");

        let result_after_commit2 = git_status(directory);
        let result_after_commit2_ = "On branch master\nYour branch is up to date with 'origin/master'.\n\nnothing to commit, working tree clean\n";
        assert_eq!(result_after_commit2, Ok(result_after_commit2_.to_string()));

        let testfile = format!("{}/{}", directory, "testfile.rs");
        fs::remove_file(testfile).expect("Error al intentar remover el archivo");

        let result_after_remove = git_status(directory);
        let result_after_remove_ = "On branch master\nChanges not staged for commit:\n  (use \"git add <file>...\" to update what will be committed)\n  (use \"git checkout -- <file>...\" to discard changes in working directory)\n\n\tdeleted:\t\ttestfile.rs\n\nno changes added to commit (use \"git add\" and/or \"git commit -a\")\n";
        assert_eq!(result_after_remove, Ok(result_after_remove_.to_string()));

        fs::remove_dir_all(directory).expect("Error al intentar remover el directorio");

        assert!(result_before_add.is_ok());
        assert!(result_after_add.is_ok());
        assert!(result_after_commit.is_ok());
        assert!(result_after_commit2.is_ok());
        assert!(result_after_remove.is_ok());
    }

    #[test]
    fn skip_gitignore_files_status() {
        let directory = "./test_status_skips_gitignore_files";
        git_init(directory).expect("Error al ejecutar git init");

        let gitignore_path = format!("{}/.gitignore", directory);
        create_file_replace(&gitignore_path, "target/\nCargo.lock\n").expect("Error al crear el archivo");

        let file_path = format!("{}/{}", directory, "testfile.rs");
        let mut file = fs::File::create(&file_path).expect("Falló al crear el archivo");
        file.write_all(b"Hola Mundo")
            .expect("Error al escribir en el archivo");
        
        let file_path2 = format!("{}/{}", directory, "Cargo.lock");
        let mut file = fs::File::create(&file_path2).expect("Falló al crear el archivo");
        file.write_all(b"Chau Mundo")
            .expect("Error al escribir en el archivo");

        let result = git_status(directory);
        let expected_result = "On branch master\nUntracked files:\n  (use \"git add <file>...\" to include in what will be committed)\n\n\tnew file: \ttestfile.rs\n\nnothing added to commit but untracked files present (use \"git add\" to track)\n";
        assert_eq!(result, Ok(expected_result.to_string()));

        fs::remove_dir_all(directory).expect("Error al intentar remover el directorio");
        assert!(result.is_ok());
    }
}
