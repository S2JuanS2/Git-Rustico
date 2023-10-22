//use crate::util::formats::decompression_object;
use std::fs::File;
use std::io::Read;

use crate::errors::GitError;

/// Esta función se utiliza para mostrar el contenido o información sobre los objetos (archivos, commits, etc.)
/// ###Parametros:
/// 'directory': dirección donde se encuentra inicializado el repositorio.
/// 'object_hash': Valor hash de 40 caracteres (SHA-1) del objeto a leer.
pub fn git_cat_file(directory: &str, object_hash: &str) -> Result<(), GitError> {
    if object_hash.len() != 40 {
        return Err(GitError::HashObjectInvalid);
    }

    //Lee los primeros 2 digitos del hash contenidos en el nombre de la carpeta.
    let path = format!("{}/.git/objects/{}", directory, &object_hash[..2]);
    //Lee los demás digitos del hash contenidos en el nombre del archivo.
    let file_path = format!("{}/{}", path, &object_hash[2..]);

    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => return Err(GitError::OpenFileError),
    };

    let mut compressed_data: Vec<u8> = vec![];

    match file.read_to_end(&mut compressed_data) {
        Ok(_) => (),
        Err(_) => return Err(GitError::ReadFileError),
    };

    //let content = decompression_object(&mut compressed_data)?;

    //println!("{}", content);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::path::Path;

    const TEST_DIRECTORY: &str = "./test_repo";

    #[test]
    fn test_git_cat_file() {
        // Se Crea un archivo simulando el contenido del objeto
        let object_hash = "0123456789abcdef0123456789abcdef01234567";
        let object_content = "test-content";

        // Se Crea un directorio temporal para el test
        let object_path = format!("{}/.git/objects/{}", TEST_DIRECTORY, &object_hash[..2]);
        fs::create_dir_all(&object_path).expect("Falló al crear el directorio temporal");

        let file_path = format!("{}/{}", object_path, &object_hash[2..]);

        let mut file = File::create(&file_path).expect("falló al crear el archivo");
        file.write_all(object_content.as_bytes())
            .expect("Falló al escribir en el archivo");

        // Cuando llama a la función git_cat_file
        let result = git_cat_file(TEST_DIRECTORY, object_hash);

        // Limpia el archivo de prueba
        if !Path::new(TEST_DIRECTORY).exists(){
            fs::remove_dir_all(TEST_DIRECTORY).expect("Falló al remover el directorio temporal");
        }

        // Deberia no devolver un Error
        assert!(result.is_ok());
    }
}
