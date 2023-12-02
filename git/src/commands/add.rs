use crate::consts::*;
use crate::models::client::Client;
use crate::util::files::{create_file_replace, open_file, read_file, read_file_string};
use crate::util::objects::builder_object_blob;
use std::ffi::OsString;
use std::fs;
use std::path::Path;

use super::check_ignore::{check_gitignore, get_gitignore_content};
use super::errors::CommandsError;
use super::rm::remove_from_index_with_filename;
use super::status::is_files_to_delete;

/// Esta función se encarga de llamar al comando add con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función add
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_add(args: Vec<&str>, client: Client) -> Result<String, CommandsError> {
    if args.len() != 1 {
        return Err(CommandsError::InvalidArgumentCountAddError);
    }
    let directory = client.get_directory_path();
    let repo_parts: Vec<&str> = directory.split('/').collect();
    let file_name = args[0];
    if args[0] != ALL {
        git_add(directory, file_name)
    } else {
        git_add_all(Path::new(directory), repo_parts.len())
    }
}

/// Esta función crea todos los objetos y los guarda
/// ###Parametros:
/// 'directory': directorio donde estará inicializado el repositorio
pub fn git_add_all(directory: &Path, repo_parts: usize) -> Result<String, CommandsError> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(_) => return Err(CommandsError::ReadDirError),
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => return Err(CommandsError::DirEntryError),
        };

        let file_name = entry.file_name();
        let full_path = entry.path();

        if full_path.is_file() {
            add_file(&full_path, &file_name, repo_parts)?;
        } else if full_path.is_dir() {
            let path_str = file_name.to_str().ok_or(CommandsError::PathToStringError)?;
            if !path_str.starts_with('.') {
                git_add_all(&full_path, repo_parts)?;
            }
        }
    }
    Ok("Files added successfully".to_string())
}

/// Esta función se encarga de llamar a la función git_add con los parametros necesarios si se hace git
/// add . y la entrada es un archivo.
/// ###Parametros:
/// 'full_path': PathBuf que contiene el path completo del archivo
/// 'file_name': Nombre del archivo que se le hizo add
fn add_file(full_path: &Path, file_name: &OsString, repo_parts: usize) -> Result<(), CommandsError> {
    let full_path_str = full_path.to_str().ok_or(CommandsError::PathToStringError)?;
    let parts: Vec<&str> = full_path_str.split('/').collect();
    let mut directory = String::new();
    let mut count = 0;
    for part in &parts {
        directory.push_str(part);
        directory.push('/');
        count += 1;
        if count == repo_parts {
            break;
        }
    }
    if parts.len() >= 3 {
        let mut dir_format = String::new();
        for part in parts.iter().take(parts.len() - 1).skip(repo_parts) {
            let dir_format_parts = format!("{}/", part);
            dir_format = dir_format + &dir_format_parts;
        }
        let file_name_str = file_name.to_string_lossy().to_string();
        let file_result = format!("{}{}", dir_format, file_name_str);
        git_add(&directory, &file_result)?;
    } else {
        let file_name_str = file_name.to_string_lossy().to_string();
        git_add(&directory, &file_name_str)?;
    }
    Ok(())
}

/// Esta función crea el objeto y lo guarda
/// ###Parametros:
/// 'directory': directorio donde estará inicializado el repositorio
/// 'file_name': Nombre del archivo del cual se leera el contenido para luego comprimirlo y generar el objeto
pub fn git_add(directory: &str, file_name: &str) -> Result<String, CommandsError> {

    if !is_files_to_delete(directory, file_name)? {
        let file_path = format!("{}/{}", directory, file_name);
        let mut ignored_files = Vec::<String>::new();
        let gitignore_content = get_gitignore_content(directory)?;
        check_gitignore(file_name, &mut ignored_files, &gitignore_content)?;
        if !ignored_files.is_empty() {
            let error_format = format!("This file {} is in .gitignore", file_name);
            return Ok(error_format);
        }
        let file = open_file(&file_path)?;
        let content = read_file(file)?;
    
        let git_dir = format!("{}/{}", directory, GIT_DIR);
    
        let hash_object = builder_object_blob(content, &git_dir)?;
    
        // Se actualiza el index.
        add_to_index(git_dir, file_name, hash_object)?;
    }else{
        remove_from_index_with_filename(directory, file_name)?;
    }

    let ok_format = format!("File {} added successfully", file_name);
    Ok(ok_format)
}

/// Esta función se encarga de actualizar el index con el nuevo archivo al que se le hizo add.
/// ###Parametros:
/// 'git_dir': directorio donde esta el directory/.git
/// 'file_name': Nombre del archivo que se le hizo add
/// 'hash_object': Hash del objeto que se creó al hacer add
pub fn add_to_index(git_dir: String, file_name: &str, hash_object: String) -> Result<(), CommandsError> {
    let index_path = format!("{}/{}", &git_dir, INDEX);

    let index_file = open_file(index_path.as_str())?;
    let index_content = read_file_string(index_file)?;

    let mut lines: Vec<String> = index_content.lines().map(String::from).collect();
    let mut updated = false;

    for line in &mut lines {
        if line.starts_with(file_name) {
            *line = format!("{} {} {}", file_name, BLOB, hash_object);
            updated = true;
            break;
        }
    }

    if !updated {
        lines.push(format!("{} {} {}", file_name, BLOB, hash_object));
    }

    let updated_index_content = lines.join("\n");
    create_file_replace(index_path.as_str(), updated_index_content.as_str())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::commands::{init::git_init, status::get_index_content};

    use super::*;

    #[test]
    fn add_test() {
        let directory = "./test_add";
        git_init(directory).expect("Error al inicializar el repositorio");

        let git_dir = format!("{}/{}", directory, GIT_DIR);
        let index_content_before_add = get_index_content(&git_dir).expect("Error al leer el index");
        assert_eq!(index_content_before_add, "");

        let file_path = format!("{}/{}", directory, "filetoadd.txt");
        let mut file = fs::File::create(&file_path).expect("Falló al crear el archivo");
        file.write_all(b"Archivo a agregar")
            .expect("Error al escribir en el archivo");

        let result = git_add(directory, "filetoadd.txt");

        let index_content_after_add = get_index_content(&git_dir).expect("Error al leer el index");
        assert_eq!(
            index_content_after_add,
            "filetoadd.txt blob 442bce82428f3a03efaa6edac44dcede0e1bd456"
        );

        fs::remove_dir_all(directory).expect("Error al eliminar el directorio");
        assert!(result.is_ok());
    }

    #[test]
    fn skip_gitignore_files_add() {
        let directory = "./test_add_skips_gitignore_files";
        git_init(directory).expect("Error al inicializar el repositorio");

        let gitignore_path = format!("{}/.gitignore", directory);
        create_file_replace(&gitignore_path, "target/\nCargo.lock\n").expect("Error al crear el archivo");

        let file_path = format!("{}/{}", directory, "filetoadd.txt");
        let mut file = fs::File::create(&file_path).expect("Falló al crear el archivo");
        file.write_all(b"Archivo a agregar")
            .expect("Error al escribir en el archivo");

        let file_path2 = format!("{}/{}", directory, "Cargo.lock");
        let mut file2 = fs::File::create(&file_path2).expect("Falló al crear el archivo");
        file2
            .write_all(b"Cargo lock")
            .expect("Error al escribir en el archivo");

        let result = git_add(directory, "filetoadd.txt");
        let git_dir = format!("{}/{}", directory, GIT_DIR);
        let index_content = get_index_content(&git_dir).expect("Error al leer el index");

        assert_eq!(index_content, "filetoadd.txt blob 442bce82428f3a03efaa6edac44dcede0e1bd456");

        let result_2 = git_add(directory, "Cargo.lock");
        let index_content_2 = get_index_content(&git_dir).expect("Error al leer el index");

        // Chequeo que no agrego Cargo.lock al index
        assert_eq!(index_content_2, "filetoadd.txt blob 442bce82428f3a03efaa6edac44dcede0e1bd456");

        fs::remove_dir_all(directory).expect("Error al eliminar el directorio");
        assert!(result.is_ok());
        assert!(result_2.is_ok());
    }
}