// use crate::errors::GitError;
use crate::util::formats::hash_generate;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;

const GIT_DIR: &str = "/.git";
const HEAD_FILE: &str = "HEAD";
const OBJECTS_DIR: &str = "objects";

fn get_head_branch(directory: &str) -> io::Result<String> {
    // "directory/.git/HEAD"
    let directory_git = format!("{}{}", directory, GIT_DIR);
    let head_file_path = Path::new(&directory_git).join(HEAD_FILE);

    let head_file = File::open(head_file_path);
    let mut head_file = match head_file {
        Ok(file) => file,
        Err(_) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "No se pudo abrir el archivo HEAD",
            ))
        }
    };
    let mut head_branch: String = String::new();
    let read_head_file = head_file.read_to_string(&mut head_branch);
    let _ = match read_head_file {
        Ok(file) => file,
        Err(_) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "No se pudo leer el archivo HEAD",
            ))
        }
    };
    let head_branch_name = head_branch.split('/').last();
    let head_branch_name = match head_branch_name {
        Some(name) => name,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "No se pudo obtener el nombre de la rama",
            ))
        }
    };
    let head_branch_name = head_branch_name.trim().to_string();

    Ok(head_branch_name)
}

pub fn print_head(directory: &str) -> io::Result<()> {
    let head_branch_name = get_head_branch(directory);
    let head_branch_name = match head_branch_name {
        Ok(name) => name,
        Err(_) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "No se pudo obtener el nombre de la rama para imprimir",
            ))
        }
    };
    println!("On branch {}", head_branch_name);
    Ok(())
}

pub fn check_hash(directory: &str) -> io::Result<()> {
    // "directory/.git"
    let directory_git = format!("{}{}", directory, GIT_DIR);

    // Creo un hashmap para guardar los archivos del directorio de trabajo y sus hashes correspondientes
    let mut working_directory_hash_list: HashMap<String, String> = HashMap::new();
    let working_directory = format!("{}{}", directory, "/git/src");
    let visit_working_directory =
        calculate_directory_hashes(&working_directory, &mut working_directory_hash_list);
    match visit_working_directory {
        Ok(file) => file,
        Err(_) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "No se pudo recorrer el directorio de trabajo",
            ))
        }
    };

    println!("working_directory_hash_list: {:?}", working_directory_hash_list);

    // Leo los archivos de objects
    // "directory/.git/objects"
    let objects_dir = Path::new(&directory_git).join(OBJECTS_DIR);
    let mut objects_hash_list: Vec<String> = Vec::new();
    let visit_objects = visit_dirs(&objects_dir, &mut objects_hash_list);
    match visit_objects {
        Ok(file) => file,
        Err(_) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "No se pudo recorrer el directorio objects",
            ))
        }
    };

    // Comparo los hashes de mis archivos con los de objects para crear un vector con los archivos que se modificaron
    let mut updated_files_list: Vec<String> = Vec::new();
    for hash in &working_directory_hash_list {
        if objects_hash_list.contains(hash.1) {
            println!(
                "hash en objects list: {:?} ubicado en archivo: {}",
                hash.1, hash.0
            );
        } else {
            updated_files_list.push(hash.0.to_string());
        }
    }

    println!("updated_files_list: {:?}", updated_files_list);

    // Si el vector de archivos modificados esta vacio, significa que no hay cambios
    if updated_files_list.is_empty() {
        let head_branch_name = get_head_branch(directory);
        let head_branch_name = match head_branch_name {
            Ok(name) => name,
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "No se pudo obtener el nombre de la rama correctamente",
                ))
            }
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
    Ok(())
}

fn visit_dirs(dir: &Path, hash_list: &mut Vec<String>) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let visit = visit_dirs(&path, hash_list);
                match visit {
                    Ok(file) => file,
                    Err(_) => {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            "No se pudo recorrer el directorio objects del path",
                        ))
                    }
                };
            } else {
                let hash_first_part = dir.file_name();
                let hash_first_part = match hash_first_part {
                    Some(name) => {
                        let name_str = name.to_str();
                        match name_str {
                            Some(name_str) => name_str,
                            None => {
                                return Err(io::Error::new(
                                    io::ErrorKind::Other,
                                    "No se pudo convertir a str la primera parte del hash",
                                ))
                            }
                        }
                    }
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            "No se pudo obtener la primera parte del hash",
                        ))
                    }
                };

                let hash_second_part = path.file_name();
                let hash_second_part = match hash_second_part {
                    Some(name) => {
                        let name_str = name.to_str();
                        match name_str {
                            Some(name_str) => name_str,
                            None => {
                                return Err(io::Error::new(
                                    io::ErrorKind::Other,
                                    "No se pudo convertir a str la segunda parte del hash",
                                ))
                            }
                        }
                    }
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            "No se pudo obtener la segunda parte del hash",
                        ))
                    }
                };
                let hash = format!("{}{}", hash_first_part, hash_second_part);
                hash_list.push(hash);
            }
        }
    }
    Ok(())
}

pub fn calculate_directory_hashes(
    directory: &str,
    hash_list: &mut HashMap<String, String>,
) -> Result<(), io::Error> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(_) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "No se pudo abrir el directorio",
            ))
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "No se pudo obtener la entrada del directorio",
                ))
            }
        };
        let path = entry.path();

        if path.is_dir() {
            let direct = match path.to_str() {
                Some(direct) => direct,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "No se pudo convertir el path a str",
                    ))
                }
            };
            match calculate_directory_hashes(direct, hash_list) {
                Ok(_) => {}
                Err(_) => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "No se pudo calcular el hash del directorio",
                    ))
                }
            };
        } else {
            let file_name = match path.to_str() {
                Some(file_name) => file_name,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "No se pudo convertir el path a str",
                    ))
                }
            };
            let file_content = match fs::read_to_string(&path) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!(
                        "Error al leer el contenido del archivo {}: {:?}",
                        file_name, e
                    );
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
    use std::env;

    #[test]
    fn test_git_status() {
        let binding = env::current_dir().expect("No se puede obtener el directorio actual");
        let current_dir = binding.to_str().unwrap();
        let current_dir = current_dir.replace("/git", "");

        assert!(print_head(&current_dir).is_ok());
        assert!(check_hash(&current_dir).is_ok());
    }
}
