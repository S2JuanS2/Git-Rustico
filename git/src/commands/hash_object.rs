use crate::errors::GitError;
use crate::util::formats::hash_generate;

use std::{fs::File, io::Read};

const BLOB: &str = "blob";
const TREE: &str = "tree";
const COMMIT: &str = "commit";

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

    let object_contents: String;

    match type_object {
        BLOB => {
            object_contents = format!(
                "{} {}\0{}",
                BLOB,
                content.len(),
                String::from_utf8_lossy(&content)
            );
        }
        TREE => {
            object_contents = format!(
                "{} {}\0{}",
                TREE,
                content.len(),
                String::from_utf8_lossy(&content)
            );
        }
        COMMIT => {
            object_contents = format!(
                "{} {}\0{}",
                COMMIT,
                content.len(),
                String::from_utf8_lossy(&content)
            );
        }
        _ => {
            object_contents = format!(
                "{} {}\0{}",
                BLOB,
                content.len(),
                String::from_utf8_lossy(&content)
            );
        }
    }

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
