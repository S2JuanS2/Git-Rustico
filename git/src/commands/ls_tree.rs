use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::files::{open_file, read_file_string};
use std::fs;

use super::cat_file::git_cat_file;


/// Esta función se encarga de llamar a al comando ls-tree con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función ls-tree
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_ls_tree(args: Vec<&str>, client: Client) -> Result<String, GitError> {
    if args.len() > 1 {
        return Err(GitError::InvalidArgumentCountLsTreeError);
    }
    let directory = client.get_directory_path();
    git_ls_tree(directory, args[0])
}

/// Lista el contenido de un arbol pasado por parametro como tree-ish.
/// ###Parametros:
/// 'directory': directorio del repositorio local.
/// 'tree_ish': un tree hash, un commit hash o un path a una branch que contiene un commit hash..
pub fn git_ls_tree(directory: &str, tree_ish: &str) -> Result<String, GitError> {
    let mut tree_hash = tree_ish.to_string();
    let directory_tree = format!("{}/.git/{}", directory, tree_ish);
    if fs::metadata(&directory_tree).is_ok() {
        tree_hash = associated_commit(directory, &directory_tree)?;
    }
    if git_cat_file(directory, &tree_hash, "-t")? == "blob" {
        return Err(GitError::InvalidTreeHashError);
    }
    if git_cat_file(directory, &tree_hash, "-t")? == "commit" {
        tree_hash = associated_tree(directory, tree_hash)?;
    }

    let mut formatted_result = String::new();
    let content_tree = git_cat_file(directory, &tree_hash, "-p")?;
    formatted_result.push_str(content_tree.as_str());

    Ok(formatted_result)
}


/// Obtiene el commit asociado a un path a una branch o a HEAD.
/// ###Parametros:
/// 'directory': directorio del repositorio local.
/// 'path_to_commit': un path a una branch (o a HEAD) que contiene un commit hash.
fn associated_commit(directory: &str, path_to_commit: &str) -> Result<String, GitError> {
    let path = open_file(path_to_commit)?;
    let content = read_file_string(path)?;
    if path_to_commit.contains("HEAD") {
        let tree = get_head_tree(directory, content)?;
        return Ok(tree);
    }
    if content.len() != 40 {
        return Err(GitError::InvalidTreeHashError);
    }
    let tree = associated_tree(directory, content)?;

    Ok(tree)
}

/// Obtiene el tree asociado a un commit hash.
/// ###Parametros:
/// 'directory': directorio del repositorio local.
/// 'content': commit hash.
fn associated_tree(directory: &str, content: String) -> Result<String, GitError> {
    let content_commit = git_cat_file(directory, &content, "-p")?;
    let parts: Vec<&str> = content_commit.split_whitespace().collect();
    let tree = parts[1];
    Ok(tree.to_string())
}

/// Obtiene el tree asociado a HEAD.
/// ###Parametros:
/// 'directory': directorio del repositorio local.
/// 'content': contenido del archivo HEAD.
fn get_head_tree(directory: &str, content: String) -> Result<String, GitError> {
    let path_branch = content.split_whitespace().collect::<Vec<&str>>()[1];
    let path_branch = format!("{}/.git/{}", directory, path_branch);
    let branch = open_file(&path_branch)?;
    let commit_branch = read_file_string(branch)?;
    let tree = associated_tree(directory, commit_branch)?;
    Ok(tree)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::commit::Commit;
    use crate::commands::init::git_init;
    use crate::commands::{add::git_add, commit::git_commit};
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_git_ls_tree() {
        let directory = "./test_ls_tree";
        git_init(directory).expect("Error al crear el repositorio");

        let file_path = format!("{}/{}", directory, "file1.rs");
        let mut file = fs::File::create(&file_path).expect("Falló al crear el archivo");
        file.write_all(b"Hola Mundo file1")
            .expect("Error al escribir en el archivo");

        let file_path2 = format!("{}/{}", directory, "file2.rs");
        let mut file2 = fs::File::create(&file_path2).expect("Falló al crear el archivo");
        file2
            .write_all(b"Hola Mundo file2")
            .expect("Error al escribir en el archivo");

        git_add(directory, "file1.rs").expect("Error al agregar el archivo");
        git_add(directory, "file2.rs").expect("Error al agregar el archivo");

        let test_commit1 = Commit::new(
            "prueba".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
            "Valen".to_string(),
            "vlanzillotta@fi.uba.ar".to_string(),
        );

        git_commit(directory, test_commit1).expect("Error al ejecutar el comando");

        // para obtener el tree del commit y ver si funciona ls-tree
        let branch_path = format!("{}/.git/refs/heads/master", directory);
        let branch = open_file(&branch_path).expect("Error al abrir el archivo");
        let commit_hash = read_file_string(branch).expect("Error al leer el archivo");
        let tree_hash = associated_tree(directory, commit_hash).expect("Error al obtener el tree");
        let result_tree = git_ls_tree(directory, &tree_hash);

        // para obtener el tree asociado al HEAD y ver si funciona ls-tree
        let result_head = git_ls_tree(directory, "HEAD");

        // para obtener el tree asociado a una branch (master) y ver si funciona ls-tree
        let result_master = git_ls_tree(directory, "refs/heads/master");

        fs::remove_dir_all(directory).expect("Error al intentar remover el directorio");

        assert!(result_tree.is_ok());
        assert!(result_head.is_ok());
        assert!(result_master.is_ok());
    }
}
