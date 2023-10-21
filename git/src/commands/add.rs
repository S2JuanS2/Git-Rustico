//use crate::commands::hash_object::calculate_hash;
use crate::errors::GitError;
use crate::util::formats::compressor_object;
use std::fs;
use std::{fs::File, io::Read};

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

    // NO ESTARIA GENERANDO BIEN EL HASH DESPUES SE ARREGLARÁ.
    //let hash_object =calculate_hash(store.as_bytes());
    let hash_object = "ABCDEF1234567891234567801234567891ABCEDF";

    //CAMBIAR POR .git
    let git_dir = format!("{}test", directory);
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

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_test() {
        fs::create_dir_all("./test/objects").expect("Error");

        let result = git_add("./", "testfile");

        fs::remove_dir_all("./test").expect("Falló al remover el directorio temporal");
        assert!(result.is_ok());
    }
}
