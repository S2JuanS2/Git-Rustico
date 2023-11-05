use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::files::{create_file_replace, open_file, read_file_string};
use std::fs;

use super::branch::get_current_branch;
use super::checkout::git_checkout_switch;
use super::cat_file::git_cat_file;
use super::log::git_log;

/// Esta función se encarga de llamar al comando merge con los parametros necesarios.
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función merge
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_merge(args: Vec<&str>, client: Client) -> Result<String, GitError> {
    if args.len() != 1 {
        return Err(GitError::InvalidArgumentCountMergeError);
    }
    let directory = client.get_directory_path();
    let branch_name = args[0];
    git_merge(directory, branch_name)
}

/// Ejecuta la accion de merge en el repositorio local.
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'branch_name': nombre de la rama a mergear
pub fn git_merge(directory: &str, branch_name: &str) -> Result<String, GitError> {
    let current_branch = get_current_branch(directory)?;
    let path_current_branch = format!("{}/.git/refs/heads/{}", directory, current_branch);
    let path_branch_to_merge = format!("{}/.git/refs/heads/{}", directory, branch_name);

    let current_branch_file = open_file(&path_current_branch)?;
    let current_branch_hash = read_file_string(current_branch_file)?;
    let merge_branch_file = open_file(&path_branch_to_merge)?;
    let branch_to_merge_hash = read_file_string(merge_branch_file)?;

    let mut formatted_result = String::new();
    if current_branch_hash == branch_to_merge_hash || current_branch_hash == branch_name {
        formatted_result.push_str("Already up to date.");
        return Ok(formatted_result);
    }
    else {
        let (log_current_branch, log_merge_branch) = get_logs_from_branches(directory, branch_name, current_branch)?;
        check_if_current_is_up_to_date(&log_current_branch, &log_merge_branch, &mut formatted_result);

        let first_commit_current_branch = &log_current_branch[0];
        let first_commit_merge_branch = &log_merge_branch[0];
        let root_parent_current_branch = git_cat_file(directory, first_commit_current_branch, "-p")?;
        let root_parent_merge_branch = git_cat_file(directory, first_commit_merge_branch, "-p")?;
        let (hash_parent_current, hash_parent_merge) = get_parent_hashes(root_parent_current_branch, root_parent_merge_branch);
        let strategy = merge_depending_on_strategy(&hash_parent_current, &hash_parent_merge, log_merge_branch, directory, branch_name)?;
        get_result_depending_on_strategy(strategy, &mut formatted_result, current_branch_hash, branch_to_merge_hash, path_current_branch)?;
    }
    Ok(formatted_result)
}

/// Obtiene los logs de las ramas a mergear.
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'branch_name': nombre de la rama a mergear
/// 'current_branch': nombre de la rama actual
fn get_logs_from_branches(directory: &str, branch_name: &str, current_branch: String) -> Result<(Vec<String>, Vec<String>), GitError> {
    let log_current_branch = git_log(directory)?;
    let log_current_branch = get_commits_from_log(log_current_branch);
    git_checkout_switch(directory, branch_name)?;
    let log_merge_branch = git_log(directory)?;
    let log_merge_branch = get_commits_from_log(log_merge_branch);
    git_checkout_switch(directory, &current_branch)?;
    Ok((log_current_branch, log_merge_branch))
}

/// Obtiene el resultado del merge dependiendo de la estrategia de merge utilizada.
/// ###Parametros:
/// 'strategy': tupla que contiene la estrategia de merge utilizada y el archivo en conflicto (u ok si no hay
/// conflictos)
/// 'formatted_result': String que contiene el resultado formateado del merge.
/// 'current_branch_hash': hash del commit de la rama actual
/// 'branch_to_merge_hash': hash del commit de la rama a mergear
/// 'path_current_branch': path del archivo de la rama actual
fn get_result_depending_on_strategy(strategy: (String, String), formatted_result: &mut String, current_branch_hash: String, branch_to_merge_hash: String, path_current_branch: String) -> Result<(), GitError> {
    if strategy.0 == "recursive" && strategy.1 == "ok" {
        formatted_result.push_str("Merge made by the 'recursive' strategy.");
    }
    else if strategy.0 == "fast-forward" {
        formatted_result.push_str(format!("Updating {}..{}\n", current_branch_hash, branch_to_merge_hash).as_str());
        formatted_result.push_str("Fast-forward\n");
        create_file_replace(&path_current_branch, &branch_to_merge_hash)?;
    }
    else {
        formatted_result.push_str(format!("Auto-merging {}\n", path_current_branch).as_str());
        formatted_result.push_str(format!("CONFLICT (content): Merge conflict in {}\n", path_current_branch).as_str());
        formatted_result.push_str("Automatic merge failed; fix conflicts and then commit the result.\n");
        formatted_result.push_str(format!("Conflict in file:{}\n", strategy.1).as_str());
    }
    Ok(())
}

/// Obtiene los hashes de los padres de los commits de las ramas a mergear.
/// ###Parametros:
/// 'root_parent_current_branch': String que contiene el contenido del primer commit de la rama actual
/// 'root_parent_merge_branch': String que contiene el contenido del primer commit de la rama a mergear
fn get_parent_hashes(root_parent_current_branch: String, root_parent_merge_branch: String) -> (String, String) {
    let mut hash_parent_current = "0000000000000000000000000000000000000000";
    let mut hash_parent_merge = "0000000000000000000000000000000000000000";
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
    (hash_parent_current.to_string(), hash_parent_merge.to_string())
}

/// Chequea si la rama actual tiene como commit al ultimo commit de la rama a mergear. En caso de tenerlo,
/// la rama actual esta mas avanzada que la rama a mergear entonces estaria actualizada.
/// ###Parametros:
/// 'log_current_branch': Vector de strings que contiene los commits de la rama actual.
/// 'log_merge_branch': Vector de strings que contiene los commits de la rama a mergear.
/// 'formatted_result': String que contiene el resultado formateado del merge.
fn check_if_current_is_up_to_date(log_current_branch: &[String], log_merge_branch: &[String], formatted_result: &mut String) {
    for commit in log_current_branch.iter() {
        if let Some(last_hash_merge_branch) = log_merge_branch.last() {
            if commit == last_hash_merge_branch.as_str() {
                formatted_result.push_str("Already up to date.");
            }
        }
    }
}

/// Recorre los commits en la merge branch y hace el merge dependiendo de la estrategia a utilizar.
/// ###Parametros:
/// 'hash_parent_current': hash del padre del commit de la rama actual
/// 'hash_parent_merge': hash del padre del commit de la rama a mergear
/// 'log_merge_branch': Vector de strings que contiene los commits de la rama a mergear.
/// 'directory': directorio del repositorio local
/// 'branch_to_merge': nombre de la rama a mergear
fn merge_depending_on_strategy(hash_parent_current: &str, hash_parent_merge: &str, log_merge_branch: Vec<String>, directory: &str, branch_to_merge: &str) -> Result<(String, String), GitError> {
    let mut strategy: (String, String) = ("".to_string(), "".to_string());
    for commit in log_merge_branch {
        let content_commit = git_cat_file(directory, &commit, "-p")?;
        let content_tree = get_tree_of_commit(content_commit, directory)?;
        for line in content_tree.lines() {
            let file = line.split_whitespace().skip(1).take(1).collect::<String>();
            let hash_blob = line.split_whitespace().skip(2).take(1).collect::<String>();
            let content_file = git_cat_file(directory, &hash_blob, "-p")?;
            let path_file_format = format!("{}/{}", directory, file);
            if hash_parent_current == hash_parent_merge {
                // RECURSIVE STRATEGY: me falta hacer el merge commit
                if let Ok(metadata) = fs::metadata(&path_file_format) {
                    if metadata.is_file() {
                        compare_files(&path_file_format, &content_file, branch_to_merge, &mut strategy)?;
                    }
                } else {
                    create_file_replace(&path_file_format, &content_file)?;
                    strategy.0 = "recursive".to_string();
                    strategy.1 = "ok".to_string();
                };
            }
            else {
                // FAST-FORWARD STRATEGY
                create_file_replace(&path_file_format, &content_file)?;
                strategy.0 = "fast-forward".to_string();
                strategy.1 = "ok".to_string();
            }
        }
    }
    Ok(strategy)
}

/// Obtiene el contenido del arbol del commit.
/// ###Parametros:
/// 'content_commit': String que contiene el contenido del commit
/// 'directory': directorio del repositorio local
fn get_tree_of_commit(content_commit: String, directory: &str) -> Result<String, GitError> {
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
fn compare_files(path_file_format: &str, content_file: &str, branch_to_merge: &str, strategy: &mut (String, String)) -> Result<(), GitError> {
    let file = open_file(path_file_format)?;
    let content_file_local = read_file_string(file)?;
    if content_file_local != content_file {
        // CONFLICTO
        let path_conflict = check_each_line(path_file_format, content_file_local, content_file, branch_to_merge)?;
        strategy.0 = "recursive".to_string();
        strategy.1 = path_conflict;
    } 
    else {
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

/// Chequea cada linea de los archivos para ver si hay conflictos.
/// ###Parametros:
/// 'path_file_format': path del archivo a comparar
/// 'content_file_local': contenido del archivo en la rama actual
/// 'content_file': contenido del archivo en la rama a mergear
/// 'branch_to_merge': nombre de la rama a mergear
fn check_each_line(path_file_format: &str, content_file_local: String, content_file: &str, branch_to_merge: &str) -> Result<String, GitError> {
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
        }
        else {
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
    use crate::commands::add::git_add;
    use crate::commands::branch::git_branch_create;
    use crate::commands::checkout::git_checkout_switch;
    use crate::commands::commit::Commit;
    use crate::commands::{init::git_init, commit::git_commit};
    use crate::commands::merge::git_merge;
    use std::{fs::{self}, io::Write};

    #[test]
    fn git_merge_fast_forward_test(){
        
        let directory = "./test_merge_fast_forward";
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

        git_checkout_switch(directory, "master").expect("Error al cambiar de rama");

        let result = git_merge(directory, "new_branch");

        fs::remove_dir_all(directory).expect("Falló al remover el directorio temporal");

        assert!(result.is_ok());
    }

    #[test]
    fn git_merge_recursive_test(){
        let directory = "./test_merge_recursive";
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

        let file_path4 = format!("{}/{}", directory, "tocommitinotra.txt");
        let mut file = fs::File::create(&file_path4).expect("Falló al crear el archivo");
        file.write_all(b"Archivo a commitear 3")
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
        file.write_all(b"Archivo a commitear 232")
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

        let result = git_merge(directory, "new_branch");

        fs::remove_dir_all(directory).expect("Falló al remover el directorio temporal");

        assert!(result.is_ok());
    }

}