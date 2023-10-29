use crate::consts::*;
use crate::models::client::Client;
use crate::util::formats::decompression_object;

use crate::errors::GitError;

/// Esta función se encarga de llamar a al comando cat-file con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función cat-file
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_cat_file(args: Vec<&str>, client: Client) -> Result<String, GitError> {
    if args.len() != 2 {
        return Err(GitError::InvalidArgumentCountCatFileError);
    }
    if args[0] != "-t" && args[0] != "-p" {
        return Err(GitError::FlagCatFileNotRecognizedError);
    }

    let directory = client.get_directory_path();
    git_cat_file(&directory, args[1])
}

/// Esta función se utiliza para mostrar el contenido o información sobre los objetos (archivos, commits, etc.)
/// ###Parametros:
/// 'directory': dirección donde se encuentra inicializado el repositorio.
/// 'object_hash': Valor hash de 40 caracteres (SHA-1) del objeto a leer.
pub fn git_cat_file(directory: &str, object_hash: &str) -> Result<String, GitError> {
    if object_hash.len() != 40 {
        return Err(GitError::HashObjectInvalid);
    }

    //Lee los primeros 2 digitos del hash contenidos en el nombre de la carpeta.
    let path = format!("{}{}/objects/{}", directory, GIT_DIR, &object_hash[..2]);
    //Lee los demás digitos del hash contenidos en el nombre del archivo.
    let file_path = format!("{}/{}", path, &object_hash[2..]);

    let content = decompression_object(&file_path)?;

    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::formats::compressor_object;
    use std::fs;
    use std::fs::File;
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

        let file = File::create(&file_path).expect("falló al crear el archivo");

        compressor_object(object_content.to_string(), file).expect("Falló en la compresión");

        // Cuando llama a la función git_cat_file
        let result = git_cat_file(TEST_DIRECTORY, object_hash).expect("Falló el comando");

        // El contenido original deberia ser igual al descomprimido
        assert_eq!(result, object_content);

        // Limpia el archivo de prueba
        if !Path::new(TEST_DIRECTORY).exists() {
            fs::remove_dir_all(TEST_DIRECTORY).expect("Falló al remover el directorio temporal");
        }
    }
}
