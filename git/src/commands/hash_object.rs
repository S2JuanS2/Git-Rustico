use crate::errors::GitError;
use crate::util::formats::hash_generate;

use std::{fs::File, io::Read};

const BLOB: &str = "blob";
const TREE: &str = "tree";
const COMMIT: &str = "commit";

/// Esta función se encarga de llamar al comando hash-object con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función hash-object
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_hash_object(args: Vec<&str>) -> Result<(), GitError> {
    if args.len() != 3 {
        return Err(GitError::InvalidArgumentCountHashObjectError);
    }
    if args[0] != "-t" {
        return Err(GitError::FlagHashObjectNotRecognizedError);
    }
    let type_object = args[1];
    let file_name = args[2];

    git_hash_object(type_object, file_name)
}

/// Esta función devuelve el hash de un objeto según su tipo.
/// ###Parametros:
/// 'type_object': tipo del objeto, puede ser, commit, tree, blob, tag
/// 'file_name': Nombre del archivo del cual se leera el contenido para generar el hash
pub fn git_hash_object(type_object: &str, file_name: &str) -> Result<(), GitError> {
    let mut file = match File::open(file_name) {
        Ok(file) => file,
        Err(_) => return Err(GitError::OpenFileError),
    };

    let mut content = Vec::new();

    match file.read_to_end(&mut content) {
        Ok(_) => (),
        Err(_) => return Err(GitError::ReadFileError),
    }

    let object_contents = match type_object {
        BLOB => {
            format!(
                "{} {}\0{}",
                BLOB,
                content.len(),
                String::from_utf8_lossy(&content)
            )
        }
        TREE => {
            format!(
                "{} {}\0{}",
                TREE,
                content.len(),
                String::from_utf8_lossy(&content)
            )
        }
        COMMIT => {
            format!(
                "{} {}\0{}",
                COMMIT,
                content.len(),
                String::from_utf8_lossy(&content)
            )
        }
        _ => {
            format!(
                "{} {}\0{}",
                BLOB,
                content.len(),
                String::from_utf8_lossy(&content)
            )
        }
    };

    let hash = hash_generate(&object_contents);

    println!("{}", hash);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_git_hash_object() {
        let temp_file_name = "prueba.txt";

        fs::write(temp_file_name, "Chau mundo").expect("Falló al escribir en el archivo");

        let result = git_hash_object("blob", temp_file_name);

        assert!(result.is_ok());

        fs::remove_file(temp_file_name).expect("Falló al remover el archivo");
    }
}
