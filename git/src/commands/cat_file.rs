use std::io::{self, Read};
use std::fs::File;


/// Esta función se utiliza para mostrar el contenido o información sobre los objetos (archivos, commits, etc.)
/// ###Parametros:
/// 'directory': dirección donde se encuentra inicializado el repositorio.
/// 'object_hash': Valor hash de 40 caracteres (SHA-1) del objeto a leer.
pub fn git_cat_file(directory: &str, object_hash: &str) -> io::Result<()>{

    if object_hash.len() != 40{
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "El hash del objeto no tiene 40 caracteres",
        ));
    }

    //Lee los primeros 2 digitos del hash contenidos en el nombre de la carpeta.
    let path = format!("{}/.git/objects/{}",directory, &object_hash[..2]); 
    //Lee los demás digitos del hash contenidos en el nombre del archivo.
    let file_path = format!("{}/{}", path, &object_hash[2..]);

    let mut file = File::open(file_path)?;
    let mut content = String::new();

    file.read_to_string(&mut content)?;

    println!("{}",content);

    Ok(())

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::Path;
    use std::fs::{self, File};

    #[test]
    fn test_git_cat_file() {
        // Se Crea un directorio temporal para el test
        let temp_dir = Path::new("/test_git_cat_file");

        fs::create_dir(&temp_dir).expect("Failed to create temporary directory");

        // Se Crea una estructura de directorios simulando .git/objects
        let objects_path = temp_dir.join("/.git/objects");
        fs::create_dir_all(&objects_path).expect("Failed to create objects directory");

        // Se Crea un archivo simulando el contenido del objeto
        let object_hash = "0123456789abcdef0123456789abcdef01234567";
        let object_content = "Contenido-Test";

        let object_dir = objects_path.join(&object_hash[..2]);
        fs::create_dir(&object_dir).expect("Failed to create object directory");

        let object_file = object_dir.join(&object_hash[2..]);
        let mut file = File::create(&object_file).expect("Failed to create object file");
        file.write_all(object_content.as_bytes()).expect("Failed to write to object file");

        // Cuando llama a la función git_cat_file
        let result = git_cat_file(temp_dir.to_str().unwrap(), object_hash);

        // Deberia no devolver un Error
        assert!(result.is_ok());

        fs::remove_dir_all(&temp_dir).expect("Failed to remove temporary directory");
    }
}
