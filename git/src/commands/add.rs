use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::formats::{compressor_object, hash_generate};
use std::fs;
use std::{fs::File, io::Read};

/// Esta función se encarga de llamar al comando add con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función add
pub fn handle_add(args: Vec<&str>, client: Client) -> Result<(), GitError> {
    if args.len() != 1 {
        return Err(GitError::InvalidArgumentCountAddError);
    }
    let directory = client.get_directory_path();
    let file_name = args[0];
    git_add(&directory, file_name)
}

/// Esta función crea el objeto y lo guarda
/// ###Parametros:
/// 'directory': directorio donde estará inicializado el repositorio
/// 'file_name': Nombre del archivo del cual se leera el contenido para luego comprimirlo y generar el objeto
pub fn git_add(directory: &str, file_name: &str) -> Result<(), GitError> {
    let file_path = format!("{}/{}", directory, file_name);

    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => return Err(GitError::OpenFileError),
    };

    let mut content = Vec::new();

    match file.read_to_end(&mut content) {
        Ok(_) => (),
        Err(_) => return Err(GitError::ReadFileError),
    }

    let header = format!("blob {}\0", content.len());

    let store = header + &String::from_utf8_lossy(&content).to_string();

    let hash_object = hash_generate(&store);

    let git_dir = format!("{}.git", directory);
    let objects_dir = format!(
        "{}/objects/{}/{}",
        &git_dir,
        hash_object[..2].to_string(),
        hash_object[2..].to_string()
    );

    let hash_object_path = format!("{}/objects/{}/", &git_dir, hash_object[..2].to_string());

    match fs::create_dir_all(hash_object_path) {
        Ok(_) => (),
        Err(_) => return Err(GitError::OpenFileError),
    }

    let file_object = match File::create(objects_dir) {
        Ok(file_object) => file_object,
        Err(_) => return Err(GitError::OpenFileError),
    };

    compressor_object(store, file_object)?;

    //Falta el index(Staging area)

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_test() {
        fs::create_dir_all("./.git/objects").expect("Error");

        let result = git_add("./", "testfile");

        fs::remove_dir_all("./.git").expect("Falló al remover el directorio temporal");
        assert!(result.is_ok());
    }
}
