use super::files::{open_file, read_file_string};
use crate::consts::{INDEX, TREE};
use crate::errors::GitError;
use crate::util::objects::builder_object_tree;

/// Maneja el index del repositorio del cliente, lo abre y devuelve su contenido
///
/// # Argumentos
///
/// * `git_dir`: Contiene la dirección del repositorio.
///
/// # Retorno
///
/// Devuelve un `Result` que contiene un (String) en caso de éxito o un error (CommandsError) en caso de fallo.
///
pub fn open_index(git_dir: &str) -> Result<String, GitError> {
    let path_index = format!("{}/{}", git_dir, INDEX);

    let index_file = open_file(&path_index)?;
    read_file_string(index_file)
}

/// Maneja el contenido del index del repositorio del cliente, creando los tree y sub tree correspondientes.
///
/// # Argumentos
///
/// * `git_dir`: Contiene la dirección del repositorio.
/// * `index_content`: Contenido del index.
///
/// # Retorno
///
/// Devuelve un `Result` que contiene un (String) en caso de éxito o un error (CommandsError) en caso de fallo.
///
pub fn recovery_index(index_content: &str, git_dir: &str) -> Result<String, GitError>{

    let mut lineas: Vec<String> = Vec::new();

    for line in index_content.lines(){
        lineas.push(line.to_string());
    }

    lineas.sort();

    let mut tree = String::new();
    let mut sub_tree = String::new();
    let mut folder_name = String::new();

    for line in lineas {
        let parts: Vec<&str> = line.split_whitespace().collect();

        let file_name = parts[0];
        let mode = parts[1];
        let hash = parts[2];

        if file_name.contains('/') {
            let path_parts: Vec<&str> = file_name.split('/').collect();
            let new_path: Vec<&str> = path_parts.clone().into_iter().skip(1).collect();
            let new_path_str = new_path.join("/");
        
            if folder_name != path_parts[0] {

                if !sub_tree.is_empty() {
                    let hash_sub_tree = recovery_index(&sub_tree, git_dir)?;
                    let blob = format!("{} {} {}\n", folder_name, TREE, hash_sub_tree);
                    tree.push_str(&blob);
                    sub_tree.clear();
                }
                folder_name = path_parts[0].to_string();
                let sub_blob = format!("{} {} {}\n", new_path_str, mode, hash);
                sub_tree.push_str(&sub_blob);
                
            }else{
                let sub_blob = format!("{} {} {}\n", new_path_str, mode, hash);
                sub_tree.push_str(&sub_blob);
            }
        }
        if !file_name.contains('/'){
            if !sub_tree.is_empty() {
                let hash_sub_tree = recovery_index(&sub_tree, git_dir)?;
                let blob = format!("{} {} {}\n", folder_name, TREE, hash_sub_tree);
                tree.push_str(&blob);
                sub_tree.clear();
            }
            let blob = format!("{} {} {}\n", file_name, mode, hash);
            tree.push_str(&blob);

        }
    }
    if !sub_tree.is_empty() {
        let hash_sub_tree = recovery_index(&sub_tree, git_dir)?;
        let blob = format!("{} {} {}\n", folder_name, TREE, hash_sub_tree);
        tree.push_str(&blob);
        sub_tree.clear();
    }
    let tree_hash = builder_object_tree(&git_dir, &tree)?;

    Ok(tree_hash)
}

/*
#[cfg(test)]
mod tests {
    use super::{recovery_index, open_index};

    #[test]
    fn test() {
        let git_dir = format!("{}/{}", "Repository", ".git");
        let content = open_index(&git_dir).expect("Error al abrir el index");
        recovery_index(&content,&git_dir).expect("Error al recorer el index");
    }
    
}
*/