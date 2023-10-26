use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::formats::hash_generate;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;

const GIT_DIR: &str = "/.git";
const HEAD_FILE: &str = "HEAD";
const OBJECTS_DIR: &str = "objects";

/// Esta función se encarga de llamar al comando status con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función status
pub fn handle_status(args: Vec<&str>, client: Client) -> Result<(), GitError> {
    if args.is_empty() {
        return Err(GitError::InvalidArgumentCountStatusError);
    }
    let directory = client.get_directory_path();
    git_status(&directory)
}

/// Devuelve el nombre de la rama actual.
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
fn get_head_branch(directory: &str) -> Result<String, GitError> {
    // "directory/.git/HEAD"
    let directory_git = format!("{}{}", directory, GIT_DIR);
    let head_file_path = Path::new(&directory_git).join(HEAD_FILE);

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
pub fn print_head(directory: &str) -> Result<(), GitError> {
    let head_branch_name = get_head_branch(directory);
    let head_branch_name = match head_branch_name {
        Ok(name) => name,
        Err(_) => return Err(GitError::HeadBranchError),
    };
    println!("On branch {}", head_branch_name);
    Ok(())
}

/// Compara los hashes de los archivos del directorio de trabajo con los de objects e imprime el estado
/// del repositorio local, incluyendo las diferencias entre los archivos locales y los archivos que ya
/// fueron agregados al staging area.
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
pub fn git_status(directory: &str) -> Result<(), GitError> {
    // "directory/.git"
    let directory_git = format!("{}{}", directory, GIT_DIR);

    let working_directory_hash_list = match get_hashes_working_directory(directory) {
        Ok(value) => value,
        Err(value) => return value,
    };

    let objects_hash_list = match get_hashes_objects(directory_git) {
        Ok(value) => value,
        Err(value) => return value,
    };

    let updated_files_list = compare_hash_lists(working_directory_hash_list, objects_hash_list);

    if let Some(value) = print_changes(updated_files_list, directory) {
        return value;
    }
    Ok(())
}

/// Imprime los cambios que se realizaron en el repositorio local y no estan en el staging area.
/// ###Parámetros:
/// 'updated_files_list': vector con los nombres de los archivos que se modificaron.
/// 'directory': directorio del repositorio local.
fn print_changes(updated_files_list: Vec<String>, directory: &str) -> Option<Result<(), GitError>> {
    // Si el vector de archivos modificados esta vacio, significa que no hay cambios
    if updated_files_list.is_empty() {
        let head_branch_name = get_head_branch(directory);
        let head_branch_name = match head_branch_name {
            Ok(name) => name,
            Err(_) => return Some(Err(GitError::HeadBranchError)),
        };
        println!(
            "Your branch is up to date with 'origin/{}'.",
            head_branch_name
        );
    } else {
        println!("Changes not staged for commit:");
        println!("  (use \"git add <file>...\" to update what will be committed)");
        println!("  (use \"git reset HEAD <file>...\" to unstage)");
        for file in updated_files_list {
            println!("\tmodified:   {}", file);
        }
    }
    None
}

/// Compara los hashes de los archivos del directorio de trabajo con los de objects y devuelve un vector
/// con los nombres de los archivos que se modificaron.
/// ###Parámetros:
/// 'working_directory_hash_list': HashMap con los nombres de los archivos en el working directory y sus hashes.
/// 'objects_hash_list': vector con los hashes de los archivos en objects.
fn compare_hash_lists(
    working_directory_hash_list: HashMap<String, String>,
    objects_hash_list: Vec<String>,
) -> Vec<String> {
    // Comparo los hashes de mis archivos con los de objects para crear un vector con los archivos que se modificaron
    let mut updated_files_list: Vec<String> = Vec::new();
    for hash in &working_directory_hash_list {
        if !objects_hash_list.contains(hash.1) {
            updated_files_list.push(hash.0.to_string());
        }
    }
    updated_files_list
}

/// Devuelve un vector con los hashes de los archivos en objects.
/// ###Parámetros:
/// 'directory_git': directorio del repositorio local.
fn get_hashes_objects(directory_git: String) -> Result<Vec<String>, Result<(), GitError>> {
    let objects_dir = Path::new(&directory_git).join(OBJECTS_DIR);
    let mut objects_hash_list: Vec<String> = Vec::new();
    let visit_objects = visit_dirs(&objects_dir, &mut objects_hash_list);
    match visit_objects {
        Ok(file) => file,
        Err(_) => return Err(Err(GitError::VisitDirectoryError)),
    };
    Ok(objects_hash_list)
}

/// Devuelve un HashMap con los nombres de los archivos en el working directory y sus hashes correspondientes.
/// ###Parámetros:
/// 'directory': directorio del repositorio local.
fn get_hashes_working_directory(
    directory: &str,
) -> Result<HashMap<String, String>, Result<(), GitError>> {
    let mut working_directory_hash_list: HashMap<String, String> = HashMap::new();
    let working_directory = format!("{}{}", directory, "/git/src");
    let visit_working_directory =
        calculate_directory_hashes(&working_directory, &mut working_directory_hash_list);
    match visit_working_directory {
        Ok(file) => file,
        Err(_) => return Err(Err(GitError::VisitDirectoryError)),
    };
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

        if path.is_dir() {
            let direct = match path.to_str() {
                Some(direct) => direct,
                None => return Err(GitError::PathToStringError),
            };
            match calculate_directory_hashes(direct, hash_list) {
                Ok(_) => {}
                Err(_) => return Err(GitError::GetHashError),
            };
        } else {
            let file_name = match path.to_str() {
                Some(file_name) => file_name,
                None => return Err(GitError::PathToStringError),
            };
            let file_content = match fs::read_to_string(&path) {
                Ok(content) => content,
                Err(e) => {
                    println!("Error: {}", e);
                    continue;
                }
            };

            let hash = hash_generate(&file_content);
            hash_list.insert(file_name.to_string(), hash);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    const TEST_DIRECTORY: &str = "./test_repo";

    #[test]
    fn test_git_status() {
        let dir_path = TEST_DIRECTORY.to_string();
        if let Err(err) = fs::create_dir_all(&dir_path) {
            panic!("Falló al crear el repo de test: {}", err);
        }

        let repo_path = format!("{}{}", dir_path, "/git/src");
        if let Err(err) = fs::create_dir_all(&repo_path) {
            panic!("Falló al crear la carpeta 'git': {}", err);
        }

        // El hash de este archivo es: 48124d6dc3b2e693a207667c32ac672414913994
        let file_path1 = format!("{}/main.rs", repo_path);
        let mut file = fs::File::create(&file_path1).expect("Falló al crear el archivo");
        file.write_all(b"Hola Mundo")
            .expect("Error al escribir en el archivo");

        let file_path2 = format!("{}/errors.rs", repo_path);
        let mut file = fs::File::create(&file_path2).expect("Falló al crear el archivo");
        file.write_all(b"Aca habria errores")
            .expect("Error al escribir en el archivo");

        let objects_path = format!("{}{}", dir_path, "/.git/objects");
        if let Err(err) = fs::create_dir_all(&objects_path) {
            panic!("Falló al crear la carpeta 'objects': {}", err);
        }

        // Agrego en la carpeta el hash unicamente del file_path1
        let folder_path = format!("{}/{}", objects_path, "48");
        if let Err(err) = fs::create_dir_all(&folder_path) {
            panic!("Falló al crear la carpeta 'd5': {}", err);
        }

        let file_path = format!("{}/124d6dc3b2e693a207667c32ac672414913994", folder_path);
        let _ = fs::File::create(&file_path).expect("Falló al crear el archivo");

        assert!(print_head(TEST_DIRECTORY).is_ok());
        assert!(git_status(TEST_DIRECTORY).is_ok());

        // Elimina el directorio de prueba
        if fs::remove_dir_all(TEST_DIRECTORY).is_err() {
            eprintln!("Error al intentar eliminar el directorio temporal");
        }
    }
}
