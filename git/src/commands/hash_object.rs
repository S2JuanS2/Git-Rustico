use crate::errors::GitError;
extern crate sha1;

use std::{fs::File, io::Read};
use sha1::{Sha1, Digest};

const BLOB: &str = "blob";
const TREE: &str = "tree";
const COMMIT: &str = "commit";
const TAG: &str = "tag";

/// Dado un contenido, genera el valor hash
/// ###Parametros:
/// 'content': contenido del que se creará el hash
fn calculate_hash(content: &[u8]) -> String{

    let mut hasher = Sha1::new();
    hasher.update(content);
    let result = hasher.finalize();

    let hash_string_result = String::from_utf8_lossy(&result);
    hash_string_result.to_string()
}

/// Esta función devuelve el hash de un objeto según su tipo.
/// ###Parametros:
/// 'type_object': tipo del objeto, puede ser, commit, tree, blob, tag
/// 'file_name': Nombre del archivo del cual se leera el contenido para generar el hash
pub fn git_hash_object(type_object: &str, file_name: &str) -> Result<(), GitError>{
    
    let mut file = match File::open(file_name){
        Ok(file) => file,
        Err(_) => return Err(GitError::OpenFileError),
    };

    let mut content = Vec::new();

    match file.read_to_end(&mut content){
        Ok(_) => (),
        Err(_) => return Err(GitError::ReadFileError),
    }

    let mut object_contents = String::new();

    match type_object {
        BLOB => {
            object_contents = format!("{} {}\0{}",BLOB, content.len(), String::from_utf8_lossy(&content));
        }
        TREE => {

        }
        COMMIT => {

        }
        TAG => {

        }
        _ => {
            object_contents = format!("{} {}\0{}",BLOB, content.len(), String::from_utf8_lossy(&content)); //DEFAULT
        }
    }

    

    let hash = calculate_hash(object_contents.as_bytes());

    println!("{}", hash);

    Ok(())
}

#[cfg(test)]
mod tests{
    use super::*;
    use std::fs;

    #[test]
    fn test_git_hash_object() {
        let file_name = "prueba.txt";

        fs::write(file_name, "Chau mundo").expect("Failed to write to file");

        let result = git_hash_object("blob", file_name);

        assert!(result.is_ok());

        fs::remove_file(file_name).expect("Failed to remove test file");
    }
}