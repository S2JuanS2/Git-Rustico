use super::files::{open_file, read_file_string, create_file_replace};
use crate::consts::{INDEX, GIT_DIR, FILE, DIRECTORY, BLOB};
use crate::util::errors::UtilError;
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
pub fn open_index(git_dir: &str) -> Result<String, UtilError> {
    let path_index = format!("{}/{}", git_dir, INDEX);

    let index_file = open_file(&path_index)?;
    read_file_string(index_file)
}

/// Maneja el index del repositorio del cliente, vacía el contenido del mismo
///
/// # Argumentos
///
/// * `directory`: Contiene la dirección del repositorio.
///
/// # Retorno
///
/// Devuelve un error (CommandsError) en caso de fallo.
///
pub fn empty_index(directory: &str) -> Result<(), UtilError> {
    let path_index = format!("{}/{}/{}", directory, GIT_DIR, INDEX);
    create_file_replace(&path_index, "")?;
    Ok(())
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
/// Devuelve un `Result` que contiene un (String) en caso de éxito o un error (UtilError) en caso de fallo.
///
pub fn recovery_index(index_content: &str, git_dir: &str) -> Result<String, UtilError> {
    let mut lines: Vec<String> = index_content.lines().map(String::from).collect();
    lines.sort();

    let mut tree = String::new();
    let mut sub_tree = String::new();
    let mut folder_name = String::new();

    for line in lines {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let file_name = parts[0];
            let mut mode = parts[1];
            let hash = parts[2];

            if mode == BLOB{
                mode = FILE;
            }
            
            if file_name.contains('/') {
                handle_folder_entry(
                    &mut tree,
                    &mut sub_tree,
                    &mut folder_name,
                    file_name,
                    mode,
                    hash,
                    git_dir,
                )?;
            } else {
                handle_file_entry(
                    &mut tree,
                    &mut sub_tree,
                    &mut folder_name,
                    file_name,
                    mode,
                    hash,
                    git_dir,
                )?;
            }
        } else {
            return Err(UtilError::InvalidObjectLength);
        }
    }
    handle_last_subtree(&mut tree, &sub_tree, &folder_name, git_dir)?;

    let tree_hash = builder_object_tree(git_dir, &tree)?;

    Ok(tree_hash)
}

/// Maneja la separación entre la carpeta y el archivo para crear los sub-tree
///
/// # Argumentos
///
/// * `tree`: Contiene la información del tree principal
/// * `sub_tree`: Contiene la información del siguente sub-tree a crear
/// * `folder_name`: Contiene el nombre de la carpeta actual para la iteración.
///
/// # Retorno
///
/// Devuelve un `Result` que contiene un () en caso de éxito o un error (UtilError) en caso de fallo.
///
fn handle_folder_entry(
    tree: &mut String,
    sub_tree: &mut String,
    folder_name: &mut String,
    file_name: &str,
    mode: &str,
    hash: &str,
    git_dir: &str,
) -> Result<(), UtilError> {
    let path_parts: Vec<&str> = file_name.split('/').collect();
    let new_path: Vec<&str> = path_parts.clone().into_iter().skip(1).collect();
    let new_path_str = new_path.join("/");

    if folder_name != path_parts[0] {
        handle_subtree(tree, sub_tree, folder_name, git_dir)?;
        *folder_name = path_parts[0].to_string();
        let sub_blob = format!("{} {} {}\n", mode, new_path_str, hash);
        sub_tree.push_str(&sub_blob);
    } else {
        let sub_blob = format!("{} {} {}\n", mode, new_path_str, hash);
        sub_tree.push_str(&sub_blob);
    }

    Ok(())
}

/// Maneja, en caso de ser un archivo, el guardado en el tree principal
///
/// # Argumentos
///
/// * `tree`: Contiene la información del tree principal
/// * `sub_tree`: Contiene la información del siguente sub-tree a crear
/// * `folder_name`: Contiene el nombre de la carpeta actual para la iteración.
///
/// # Retorno
///
/// Devuelve un `Result` que contiene un () en caso de éxito o un error (UtilError) en caso de fallo.
///
fn handle_file_entry(
    tree: &mut String,
    sub_tree: &mut String,
    folder_name: &mut str,
    file_name: &str,
    mode: &str,
    hash: &str,
    git_dir: &str,
) -> Result<(), UtilError> {
    handle_subtree(tree, sub_tree, folder_name, git_dir)?;

    let blob = format!("{} {} {}\n", mode, file_name, hash);
    tree.push_str(&blob);

    Ok(())
}

/// Crea el sub-tree en caso de no estar vacío y lo recorre
///
/// # Argumentos
///
/// * `tree`: Contiene la información del tree principal
/// * `sub_tree`: Contiene la información del siguente sub-tree a crear
/// * `folder_name`: Contiene el nombre de la carpeta actual para la iteración.
/// * `git_dir`: Contiene la dirección del repositorio.
///
/// # Retorno
///
/// Devuelve un `Result` que contiene un () en caso de éxito o un error (UtilError) en caso de fallo.
///
fn handle_subtree(
    tree: &mut String,
    sub_tree: &mut String,
    folder_name: &str,
    git_dir: &str,
) -> Result<(), UtilError> {
    if !sub_tree.is_empty() {
        let hash_sub_tree = recovery_index(sub_tree, git_dir)?;
        let blob = format!("{} {} {}\n", DIRECTORY, folder_name, hash_sub_tree);
        tree.push_str(&blob);
        sub_tree.clear();
    }

    Ok(())
}

/// Maneja la creación del último sub-tree si lo hubiese.
///
/// # Argumentos
///
/// * `tree`: Contiene la información del tree principal
/// * `sub_tree`: Contiene la información del siguente sub-tree a crear
/// * `folder_name`: Contiene el nombre de la carpeta actual para la iteración.
///
/// # Retorno
///
/// Devuelve un `Result` que contiene un () en caso de éxito o un error (UtilError) en caso de fallo.
///
fn handle_last_subtree(
    tree: &mut String,
    sub_tree: &str,
    folder_name: &str,
    git_dir: &str,
) -> Result<(), UtilError> {
    if !sub_tree.is_empty() {
        handle_subtree(tree, &mut sub_tree.to_string(), folder_name, git_dir)?;
    }
    Ok(())
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
