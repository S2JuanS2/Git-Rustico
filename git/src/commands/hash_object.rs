use crate::util::files::read_file;
use crate::{consts::*, util::files::open_file};
use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::formats::hash_generate;

/// Esta función se encarga de llamar al comando hash-object con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función hash-object
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_hash_object(args: Vec<&str>, client: Client) -> Result<String, GitError> {
    if args.len() == 1 {
        git_hash_object_blob(args[0], client.get_directory_path())
    } else if args.len() == 3 && args[1] == BLOB {
        git_hash_object_blob(args[2], client.get_directory_path())
    } else if args.len() == 3 && args[1] == COMMIT {
        git_hash_object_blob(args[0], client.get_directory_path())
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

    let file = open_file(&path)?;
    let content = read_file(file)?;

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
    use std::fs::{self, File};

    #[test]
    fn test_git_hash_object() {
        let temp_file_name = "prueba.txt";
        let temp_dir = "./test_hash_object_repo";
        let path = format!("{}/{}", temp_dir, temp_file_name);
        fs::create_dir_all(&temp_dir).expect("Falló al crear el directorio temporal");
        File::create(&path).expect("Falló al crear el archivo");

        fs::write(&path, "Chau mundo").expect("Falló al escribir en el archivo");

        let result = git_hash_object_blob(temp_file_name, temp_dir).expect("Falló el comando");

        assert_eq!(result, "06ae662f3a48ae0354f4eaec7a03008a63b2dc4b");

        fs::remove_dir_all(temp_dir).expect("Falló al remover el archivo");
    }
}
