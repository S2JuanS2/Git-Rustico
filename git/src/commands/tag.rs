use crate::consts::{GIT_DIR, REFS_TAGS};
use super::errors::CommandsError;
use crate::models::client::Client;
use crate::util::files::{open_file, read_file_string, create_file, delete_file};
use crate::util::objects::builder_object_tag;

use super::branch::get_current_branch;

const TAG_DIR: &str = "refs/heads/";
const TAG_EDITMSG: &str = "TAG_EDITMSG";

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

//git show v1.0 -> muestra la información de la tag

/// Esta función se encarga de llamar a al comando tag con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función tag
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_tag(args: Vec<&str>, client: Client) -> Result<String, CommandsError> {
    let directory = client.get_directory_path();
    if args.is_empty() {
        git_tag(client.get_directory_path())
    }else if args.len() == 4 && args[0] == "-a" {
        git_tag_create(directory, client.clone(), args[1], args[3])
    }else if args.len() == 2 && args[0] == "-d" {
        git_tag_delete(directory, args[1])
    }else{
        return Err(CommandsError::InvalidArgumentCountTagError)
    }
}

// Devuelve un vector con los nombres de las tags
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
pub fn get_tags(directory: &str) -> Result<Vec<String>, CommandsError> {
    // "directory/.git/refs/heads"
    let directory_git = format!("{}/{}", directory, GIT_DIR);
    let tag_dir = Path::new(&directory_git).join(REFS_TAGS);

    let entries = match fs::read_dir(tag_dir) {
        Ok(entries) => entries,
        Err(_) => return Err(CommandsError::TagDirectoryOpenError),
    };

    let mut tags: Vec<String> = Vec::new();

    for entry in entries {
        match entry {
            Ok(entry) => {
                let tag = match entry.file_name().into_string() {
                    Ok(tag) => tag,
                    Err(_) => return Err(CommandsError::ReadTagsError),
                };
                tags.push(tag);
            }
            Err(_) => return Err(CommandsError::ReadTagsError),
        }
    }
    Ok(tags)
}

/// Esta función se encarga de mostrar las tags existenes el el directorio
/// uso: git tag -> muestra todas las tags
/// ###Parametros:
/// 'directory': directory del repositorio que contiene la tag
fn git_tag(directory: &str) -> Result<String, CommandsError> {

    let tags = get_tags(directory)?;
    let mut formatted_tags = String::new();
    for tag in tags {
        formatted_tags.push_str(&format!(" - {}\n", tag))
    }
    Ok(formatted_tags)
}

/// Creará el archivo donde se guarda el mensaje del commit
/// ###Parametros:
/// 'directory': Directorio del git
/// 'msg': mensaje del commit
fn builder_tag_msg_edit(directory: &str, msg: &str) -> Result<(), CommandsError> {
    let commit_msg_path = format!("{}/{}/{}", directory, GIT_DIR, TAG_EDITMSG);

    let format_content = format!("\n#\n# Write a message for tag:\n# {}\n# Lines starting with '#' will be ignored", msg);

    create_file(&commit_msg_path, &format_content)?;

    Ok(())
}

/// Ejecuta las funciones para crear el objeto Tag.
/// Uso: git tag -a v1.0 -m "Version 1.0" -> Crea nueva tag anotada
/// ###Parametros:
/// 'directory': directorio del repositorio local.
/// 'client': Cliente que crea la tag.
/// 'tag_name': nombre de la tag.
/// 'version_name': comentario de la tag.
pub fn git_tag_create(directory: &str, client: Client, tag_name: &str, version_name: &str) -> Result<String, CommandsError> {
    
    let tags = get_tags(directory)?;
    if tags.contains(&tag_name.to_string()) {
        return Err(CommandsError::TagAlreadyExistsError);
    }

    let git_dir = format!("{}/{}", directory, GIT_DIR);
    let current_branch = get_current_branch(directory)?;
    let branch_current_path = format!("{}/{}{}", git_dir, TAG_DIR, current_branch);

    let commit_hash;
    if fs::metadata(&branch_current_path).is_ok() {
        let file = open_file(&branch_current_path)?;
        commit_hash = read_file_string(file)?;
    }else{
        return Err(CommandsError::OpenFileError)
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time error")
        .as_secs();

    let tag_content = format!(
        "object {}\ntype commit\ntag {}\ntagger {} <{}> {} +0000\n\n{}",
        commit_hash,
        tag_name,
        client.get_name(),
        client.get_email(),
        timestamp,
        version_name,
    );

    let tag_hash = builder_object_tag(&tag_content, &git_dir)?;

    let dir_tag = format!("{}/.git/refs/tags/{}", directory, version_name);

    create_file(&dir_tag, &tag_hash)?;

    builder_tag_msg_edit(directory,tag_name)?;

    Ok("Tag creada con éxito".to_string())
}

/// Esta función se encarga de eliminar la tag con el nombre recibido por parámetro
/// uso: git tag -d v1.0 -> elimina la tag
/// ###Parametros:
/// 'directory': directory del repositorio que contiene la tag
/// 'tag_name_delete': nombre de la tag a eliminar
fn git_tag_delete(directory: &str, tag_name_delete: &str) -> Result<String, CommandsError> {

    let tags = get_tags(directory)?;
    if !tags.contains(&tag_name_delete.to_string()) {
        return Err(CommandsError::TagNotExistsError);
    }
    let dir_tag = format!("{}/.git/refs/tags/{}", directory, tag_name_delete);

    delete_file(&dir_tag)?;

    Ok("Eliminada con éxito".to_string())
}