use crate::consts::*;
use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::files::{open_file, read_file};
use crate::util::formats::hash_generate;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Esta función se encarga de llamar al comando status con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función status
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_status(args: Vec<&str>, client: Client) -> Result<String, GitError> {
    if !args.is_empty() {
        return Err(GitError::InvalidArgumentCountStatusError);
    }
    let directory = client.get_directory_path();
    git_status(directory)
}

/// Devuelve el nombre de la rama actual.
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
fn get_head_branch(directory: &str) -> Result<String, GitError> {
    // "directory/.git/HEAD"
    let directory_git = format!("{}/{}", directory, GIT_DIR);
    let head_file_path = Path::new(&directory_git).join(HEAD);

    let head_file = File::open(head_file_path);
    let mut head_file = match head_file {
        Ok(file) => file,
        Err(_) => return Err(GitError::OpenFileError),
    };
    let mut head_branch: String = String::new();
    let read_head_file = head_file.read_to_string(&mut head_branch);
    let _ = match read_head_file {
        Ok(file) => file,
        Err(_) => return Err(GitError::ReadFileError),
    };
    let head_branch_name = head_branch.split('/').last();
    let head_branch_name = match head_branch_name {
        Some(name) => name,
        None => return Err(GitError::HeadBranchError),
    };
    let head_branch_name = head_branch_name.trim().to_string();

    Ok(head_branch_name)
}

/// Muestra por pantalla el nombre de la rama actual.
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
pub fn print_head(directory: &str) -> Result<String, GitError> {
    let head_branch_name = get_head_branch(directory);
    let head_branch_name = match head_branch_name {
        Ok(name) => name,
        Err(_) => return Err(GitError::HeadBranchError),
    };
    Ok(head_branch_name)
}

/// Compara los hashes de los archivos del directorio de trabajo con los de objects e imprime el estado
/// del repositorio local, incluyendo las diferencias entre los archivos locales y los archivos que ya
/// fueron agregados al staging area.
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
pub fn git_status(directory: &str) -> Result<String, GitError> {
    // "directory/.git"
    let directory_git = format!("{}/{}", directory, GIT_DIR);

    let index_content = get_index_content(&directory_git)?;

    // Divide el contenido del índice en líneas.
    let lines: Vec<String> = index_content.lines().map(String::from).collect();
    let mut index_files: Vec<String> = Vec::new();
    if !lines.is_empty() {
        for line in lines {
            index_files.push(line);
        }
    }

    let working_directory_hash_list = get_hashes_working_directory(directory)?;

    let objects_hash_list = get_hashes_objects(directory_git)?;

    let untracked_files_list = compare_hash_lists(working_directory_hash_list, objects_hash_list);

    let value = print_changes(index_files, untracked_files_list, directory)?;

    Ok(value)
}

/// Devuelve el contenido del archivo index.
/// ###Parámetros:
/// 'directory_git': directorio del repositorio local.
pub fn get_index_content(directory_git: &String) -> Result<String, GitError> {
    let index_path = format!("{}/index", directory_git);
    let index_file = File::open(index_path);
    let mut index_file = match index_file {
        Ok(file) => file,
        Err(_) => return Err(GitError::OpenFileError),
    };
    let mut index_content: String = String::new();
    let read_index_file = index_file.read_to_string(&mut index_content);
    let _ = match read_index_file {
        Ok(file) => file,
        Err(_) => return Err(GitError::ReadFileError),
    };
    Ok(index_content)
}

/// Imprime los cambios que se realizaron en el repositorio local y no estan en el staging area.
/// ###Parámetros:
/// 'updated_files_list': vector con los nombres de los archivos que se modificaron.
/// 'directory': directorio del repositorio local.
fn print_changes(
    index_files_list: Vec<String>,
    untracked_files_list: Vec<(String, String)>,
    directory: &str,
) -> Result<String, GitError> {
    let mut formatted_result = String::new();
    let head_branch_name = get_head_branch(directory)?;
    // Si el vector de archivos modificados esta vacio, significa que no hay cambios
    formatted_result.push_str("On branch ");
    formatted_result.push_str(&head_branch_name);
    if index_files_list.is_empty() && untracked_files_list.is_empty() {
        formatted_result.push_str(&format!(
            "Your branch is up to date with 'origin/{}'.\n",
            head_branch_name
        ));
        formatted_result.push_str("\nnothing to commit, working tree clean\n");
    }
    if !index_files_list.is_empty() {
        formatted_result.push_str("\nChanges to be committed:\n");
        formatted_result.push_str("  (use \"git reset HEAD <file>...\" to unstage)\n");

        for file in index_files_list {
            let file_name: Vec<&str> = file.split(' ').collect();
            let file_name = match file_name.first() {
                Some(name) => name,
                None => return Err(GitError::HeadBranchError), //CAMBIAR ERROR
            };
            formatted_result.push_str(&format!("\tmodified:   {}\n", file_name));
        }
    }
    if !untracked_files_list.is_empty() {
        formatted_result.push_str("\nChanges not staged for commit:\n");
        formatted_result
            .push_str("  (use \"git add <file>...\" to update what will be committed)\n");
        formatted_result.push_str(
            "  (use \"git checkout -- <file>...\" to discard changes in working directory)\n",
        );

        for file in untracked_files_list {
            let file_path = match file.0.split('/').last() {
                Some(name) => name,
                None => return Err(GitError::HeadBranchError), //CAMBIAR ERROR
            };
            formatted_result.push_str(&format!("\t{}\n", file_path));
        }
    }

    Ok(formatted_result)
}

/// Compara los hashes de los archivos del directorio de trabajo con los de objects y devuelve un vector
/// con los nombres de los archivos que se modificaron.
/// ###Parámetros:
/// 'working_directory_hash_list': HashMap con los nombres de los archivos en el working directory y sus hashes.
/// 'objects_hash_list': vector con los hashes de los archivos en objects.
fn compare_hash_lists(
    working_directory_hash_list: HashMap<String, String>,
    objects_hash_list: Vec<String>,
) -> Vec<(String, String)> {
    // Comparo los hashes de mis archivos con los de objects para crear un vector con los archivos que se modificaron
    let mut updated_files_list: Vec<(String, String)> = Vec::new();
    for hash in &working_directory_hash_list {
        if !objects_hash_list.contains(hash.1) {
            updated_files_list.push((hash.0.to_string(), hash.1.to_string()));
        }
    }
    updated_files_list
}

/// Devuelve un vector con los hashes de los archivos en objects.
/// ###Parámetros:
/// 'directory_git': directorio del repositorio local.
fn get_hashes_objects(directory_git: String) -> Result<Vec<String>, GitError> {
    let objects_dir = Path::new(&directory_git).join(DIR_OBJECTS);
    let mut objects_hash_list: Vec<String> = Vec::new();
    visit_dirs(&objects_dir, &mut objects_hash_list)?;
    Ok(objects_hash_list)
}

/// Devuelve un HashMap con los nombres de los archivos en el working directory y sus hashes correspondientes.
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
fn get_hashes_working_directory(directory: &str) -> Result<HashMap<String, String>, GitError> {
    let mut working_directory_hash_list: HashMap<String, String> = HashMap::new();
    let working_directory = directory.to_string();
    calculate_directory_hashes(&working_directory, &mut working_directory_hash_list)?;
    Ok(working_directory_hash_list)
}

/// Recorre el directorio de objects recursivamente y devuelve un vector con los hashes de los archivos alli.
/// ###Parámetros:
/// 'dir': directorio del repositorio local.
/// 'hash_list': vector con los hashes de los archivos en objects.
fn visit_dirs(dir: &Path, hash_list: &mut Vec<String>) -> Result<(), GitError> {
    if dir.is_dir() {
        let fs = match fs::read_dir(dir) {
            Ok(fs) => fs,
            Err(_) => return Err(GitError::ReadDirError),
        };
        for entry in fs {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => return Err(GitError::ReadFileError),
            };
            let path = entry.path();

            if path.is_dir() {
                let visit = visit_dirs(&path, hash_list);
                match visit {
                    Ok(file) => file,
                    Err(_) => return Err(GitError::VisitDirectoryError),
                };
            } else {
                let hash_first_part = dir.file_name();
                let hash_first_part = match hash_first_part {
                    Some(name) => {
                        let name_str = name.to_str();
                        match name_str {
                            Some(name_str) => name_str,
                            None => return Err(GitError::GetHashError),
                        }
                    }
                    None => return Err(GitError::GetHashError),
                };

                let hash_second_part = path.file_name();
                let hash_second_part = match hash_second_part {
                    Some(name) => {
                        let name_str = name.to_str();
                        match name_str {
                            Some(name_str) => name_str,
                            None => return Err(GitError::GetHashError),
                        }
                    }
                    None => return Err(GitError::GetHashError),
                };
                let hash = format!("{}{}", hash_first_part, hash_second_part);
                hash_list.push(hash);
            }
        }
    }
    Ok(())
}

/// Recorre el directorio de trabajo recursivamente y devuelve un HashMap con los nombres de los archivos y
/// sus hashes correspondientes.
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
/// 'hash_list': HashMap con los nombres de los archivos en el working directory y sus hashes.
pub fn calculate_directory_hashes(
    directory: &str,
    hash_list: &mut HashMap<String, String>,
) -> Result<(), GitError> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(_) => return Err(GitError::ReadDirError),
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => return Err(GitError::DirEntryError),
        };
        let path = entry.path();

        let file_name = entry.file_name();
        let entry = match file_name.to_str() {
            Some(entry) => entry,
            None => return Err(GitError::PathToStringError),
        };

        if entry.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            let direct = match path.to_str() {
                Some(direct) => direct,
                None => return Err(GitError::PathToStringError),
            };
            calculate_directory_hashes(direct, hash_list)?;
        } else {
            let file_name = match path.to_str() {
                Some(file_name) => file_name,
                None => return Err(GitError::PathToStringError),
            };
            let file = open_file(file_name)?;
            let content = read_file(file)?;

            let header = format!("{} {}\0", BLOB, content.len());
            let store = header + String::from_utf8_lossy(&content).as_ref();
            let hash_object = hash_generate(&store);

            hash_list.insert(file_name.to_string(), hash_object);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::commands::add::git_add;

    use super::*;
    use std::io::Write;

    const TEST_DIRECTORY: &str = "./test_repo";

    #[test]
    fn test_git_status() {
        if let Err(err) = fs::create_dir_all(&TEST_DIRECTORY) {
            panic!("Falló al crear el repo de test: {}", err);
        }

        let directory_git = format!("{}/{}", TEST_DIRECTORY, GIT_DIR);
        if let Err(err) = fs::create_dir_all(&directory_git) {
            panic!("Falló al crear la carpeta 'git': {}", err);
        }

        let head_path = format!("{}/{}/{}", TEST_DIRECTORY, GIT_DIR, HEAD);
        let mut file = fs::File::create(&head_path).expect("Falló al crear el HEAD");
        file.write_all(b"ref: refs/heads/master")
            .expect("Error al escribir en el archivo");

        let objects_path = format!("{}{}", TEST_DIRECTORY, "/.git/objects");
        if let Err(err) = fs::create_dir_all(&objects_path) {
            panic!("Falló al crear la carpeta 'objects': {}", err);
        }

        File::create(format!("{}/.git/index", TEST_DIRECTORY)).expect("Error");

        let file_path = format!("{}/{}", TEST_DIRECTORY, "testfile.rs");
        let mut file = fs::File::create(&file_path).expect("Falló al crear el archivo");
        file.write_all(b"Hola Mundo")
            .expect("Error al escribir en el archivo");

        let file_path2 = format!("{}/{}", TEST_DIRECTORY, "main.rs");
        let mut file = fs::File::create(&file_path2).expect("Falló al crear el archivo");
        file.write_all(b"Chau Mundo")
            .expect("Error al escribir en el archivo");

        assert!(git_status(TEST_DIRECTORY).is_ok());

        let _ = git_add(TEST_DIRECTORY, "testfile.rs");

        assert!(git_status(TEST_DIRECTORY).is_ok());

        fs::remove_dir_all(TEST_DIRECTORY).expect("Error al intentar remover el directorio");
    }
}