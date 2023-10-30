use crate::consts::*;
use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::formats::hash_generate;

use std::{fs::File, io::Read};

/// Esta función se encarga de llamar al comando hash-object con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función hash-object
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_hash_object(args: Vec<&str>, client: Client) -> Result<String, GitError> {
    if args.len() == 1 {
        git_hash_object_blob(args[0], client.get_directory_path().as_str())
    } else if args.len() == 3 && args[1] == BLOB {
        git_hash_object_blob(args[2], client.get_directory_path().as_str())
    } else if args.len() == 3 && args[1] == TREE {
        //directorio
        git_hash_object_blob(args[0], client.get_directory_path().as_str())
    } else if args.len() == 3 && args[1] == COMMIT {
        //objeto commit
        git_hash_object_blob(args[0], client.get_directory_path().as_str())
    } else {
        return Err(GitError::InvalidArgumentCountHashObjectError);
    }
}

/// Esta función devuelve el hash de un objeto según su tipo.
/// ###Parametros:
/// 'type_object': tipo del objeto, puede ser, commit, tree, blob, tag
/// 'file_name': Nombre del archivo del cual se leera el contenido para generar el hash
pub fn git_hash_object_blob(file_name: &str, directory: &str) -> Result<String, GitError> {
    let path = format!("{}/{}", directory, file_name);

    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return Err(GitError::OpenFileError),
    };

    let mut content = Vec::new();

    match file.read_to_end(&mut content) {
        Ok(_) => (),
        Err(_) => return Err(GitError::ReadFileError),
    }

    let object_contents = format!(
        "{} {}\0{}",
        BLOB,
        content.len(),
        String::from_utf8_lossy(&content)
    );

    let hash = hash_generate(&object_contents);

    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_git_hash_object() {
        let temp_file_name = "prueba.txt";
        let path = format!("{}/{}", "test_repo", temp_file_name);
        File::create(&path).expect("Falló al crear el archivo");

        fs::write(&path, "Chau mundo").expect("Falló al escribir en el archivo");

        let result = git_hash_object_blob(temp_file_name, "test_repo").expect("Falló el comando");

        assert_eq!(result, "06ae662f3a48ae0354f4eaec7a03008a63b2dc4b");

        fs::remove_dir_all("test_repo").expect("Falló al remover el archivo");
    }
}
