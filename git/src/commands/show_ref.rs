use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::files::{open_file, read_file_string};
use std::fs;

/// Esta función se encarga de llamar a al comando ls-files con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función ls-files
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_show_ref(args: Vec<&str>, client: Client) -> Result<String, GitError> {
    if !args.is_empty() {
        return Err(GitError::InvalidArgumentShowRefError);
    }
    let directory = client.get_directory_path();
    git_show_ref(directory)
}

/// Muestra las referencias de un repositorio local con sus commits.
/// ###Parametros:
/// 'directory': directorio del repositorio local.
pub fn git_show_ref(directory: &str) -> Result<String, GitError> {
    let refs_heads_path = format!("{}/.git/refs/heads", directory);
    let refs_remotes_path = format!("{}/.git/refs/remotes", directory);
    let refs_tags_path = format!("{}/.git/refs/tags", directory);
    let mut formatted_result = String::new();

    visit_refs_dirs(refs_heads_path, &mut formatted_result, directory)?;
    visit_refs_dirs(refs_remotes_path, &mut formatted_result, directory)?;
    visit_refs_dirs(refs_tags_path, &mut formatted_result, directory)?;

    Ok(formatted_result)
}

/// Recorre los directorios de .git/refs y agrega los contenidos al resultado formateado.
/// ###Parametros:
/// 'refs_path': directorio para recorrer.
/// 'formatted_result': resultado formateado.
/// 'directory': directorio del repositorio local.
fn visit_refs_dirs(
    refs_path: String,
    formatted_result: &mut String,
    directory: &str,
) -> Result<(), GitError> {
    let git_dir = format!("{}/.git/", directory);
    let refs = &refs_path[git_dir.len()..];
    if fs::metadata(&refs_path).is_ok() {
        let entries = match fs::read_dir(&refs_path) {
            Ok(entries) => entries,
            Err(_) => return Err(GitError::ReadDirError),
        };
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => return Err(GitError::GenericError),
            };
            if entry.path().is_dir() {
                if let Some(entry) = entry.path().to_str() {
                    visit_refs_dirs(entry.to_string(), formatted_result, directory)?;
                }
            } else if let Ok(file_name) = entry.file_name().into_string() {
                let file_path = format!("{}/{}", refs_path, file_name);
                let file_hash = open_file(&file_path)?;
                let file_hash_content = read_file_string(file_hash)?;
                formatted_result.push_str(
                    format!("{} {}/{}\n", file_hash_content.trim(), refs, file_name).as_str(),
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{commands::init::git_init, util::files::create_file_replace};

    #[test]
    fn test_show_ref() {
        let directory = "./test_git_show_ref";
        git_init(directory).expect("Error al inicializar el repositorio");

        let file_head = format!("{}/.git/refs/heads/master", directory);
        create_file_replace(&file_head, "18782jhbdshiu299wue2901hsd02wi982hoq8910")
            .expect("Error al crear el archivo");

        let remotes_dir = format!("{}/.git/refs/remotes", directory);
        fs::create_dir_all(remotes_dir).expect("Error al crear el directorio");
        let origin_dir = format!("{}/.git/refs/remotes/origin", directory);
        fs::create_dir_all(origin_dir).expect("Error al crear el directorio");
        let file_remote = format!("{}/.git/refs/remotes/origin/master", directory);
        create_file_replace(&file_remote, "sjlk28989d8io02100w8uo8iwendof32e33ewr03")
            .expect("Error al crear el archivo");

        let tags_dir = format!("{}/.git/refs/tags", directory);
        fs::create_dir_all(tags_dir).expect("Error al crear el directorio");
        let file_tag = format!("{}/.git/refs/tags/v0.1.0", directory);
        create_file_replace(&file_tag, "2309kh489982094hoif8402jk48209jh843f4392")
            .expect("Error al crear el archivo");

        let result = git_show_ref(directory);
        assert!(result.is_ok());

        fs::remove_dir_all(directory).expect("Error al borrar el directorio");
    }
}
