use crate::consts::*;
use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::files::{open_file, read_file};
use crate::util::objects::builder_object_blob;
use std::fs;
use std::fs::OpenOptions;
use std::path::Path;
use std::{fs::File, io::Read, io::Write};

/// Esta función se encarga de llamar al comando add con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función add
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_add(args: Vec<&str>, client: Client) -> Result<String, GitError> {
    let result;
    if args.len() != 1 {
        return Err(GitError::InvalidArgumentCountAddError);
    }
    let directory = client.get_directory_path();
    let file_name = args[0];
    if args[0] != ALL {
        result = git_add(directory, file_name)?;
    } else {
        result = git_add_all(Path::new(directory))?;
    }
    Ok(result)
}

/// Esta función crea todos los objetos y los guarda
/// ###Parametros:
/// 'directory': directorio donde estará inicializado el repositorio
pub fn git_add_all(directory: &Path) -> Result<String, GitError> {
    
    let entries = match fs::read_dir(directory){
        Ok(entries) => entries,
        Err(_) => return Err(GitError::ReadDirError),
    };

    for entry in entries {
        let entry = match entry{
            Ok(entry) => entry,
            Err(_) => return Err(GitError::DirEntryError),
        };

        let file_name = entry.file_name();
        let full_path = entry.path();
        
        if full_path.is_file() {
            let full_path_str = full_path.to_str().ok_or(GitError::PathToStringError)?;
            let parts: Vec<&str> = full_path_str.split('/').collect();
            let directory = format!("{}/", parts[0]);
            if parts.len() >= 3 {
                let mut dir_format = format!("");
                for i in 1..parts.len()-1 {
                    let dir_format_parts = format!("{}/", parts[i]);
                    dir_format = dir_format + &dir_format_parts;
                }
                let file_name_str = file_name.to_string_lossy().to_string();
                let file_result = format!("{}{}", dir_format, file_name_str);
                git_add(&directory, &file_result)?;
            }else{
                let file_name_str = file_name.to_string_lossy().to_string();
                git_add(&directory, &file_name_str)?;
            }
        }else if full_path.is_dir() {
            let path_str = file_name.to_str().ok_or(GitError::PathToStringError)?;
            if !path_str.starts_with("."){
                git_add_all(&full_path)?;
            }
        }
    }
    Ok("Archivos agregados con exito!".to_string())
}

/// Esta función crea el objeto y lo guarda
/// ###Parametros:
/// 'directory': directorio donde estará inicializado el repositorio
/// 'file_name': Nombre del archivo del cual se leera el contenido para luego comprimirlo y generar el objeto
pub fn git_add(directory: &str, file_name: &str) -> Result<String, GitError> {
    let file_path = format!("{}/{}", directory, file_name);

    let file = open_file(&file_path)?;

    let content = read_file(file)?;

    let git_dir = format!("{}/{}", directory, GIT_DIR);

    let hash_object = builder_object_blob(content, &git_dir)?;

    // Se actualiza el index.
    add_to_index(git_dir, file_name, hash_object)?;

    Ok("Archivo agregado con exito!".to_string())
}

fn add_to_index(git_dir: String, file_name: &str, hash_object: String) -> Result<(), GitError> {
    let index_path = format!("{}/{}", &git_dir, INDEX);

    // Abre el archivo de índice en modo lectura y escritura.
    let mut index_file = match OpenOptions::new().read(true).write(true).open(&index_path) {
        Ok(index_file) => index_file,
        Err(_) => return Err(GitError::OpenFileError),
    };

    // Lee el contenido del archivo de índice.
    let mut index_content = String::new();
    if index_file.read_to_string(&mut index_content).is_err() {
        return Err(GitError::ReadFileError);
    }

    // Divide el contenido del índice en líneas.
    let mut lines: Vec<String> = index_content.lines().map(String::from).collect();
    let mut updated = false;

    // Busca si el file_name ya existe en el índice.
    for line in &mut lines {
        if line.starts_with(file_name) {
            // Actualiza la línea existente con el nuevo hash_object.
            *line = format!("{} {} {}", file_name, BLOB, hash_object);
            updated = true;
            break;
        }
    }

    // Si el file_name no existía en el índice, agrega una nueva entrada.
    if !updated {
        lines.push(format!("{} {} {}", file_name, BLOB, hash_object));
    }

    // Reescribe el contenido del archivo de índice.
    let updated_index_content = lines.join("\n");
    let mut index_file = match File::create(index_path) {
        Ok(index_file) => index_file,
        Err(_) => return Err(GitError::OpenFileError),
    };
    if index_file.set_len(0).is_err() {
        return Err(GitError::WriteFileError);
    }
    if index_file
        .write_all(updated_index_content.as_bytes())
        .is_err()
    {
        return Err(GitError::WriteFileError);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_test() {
        // Se crea un directorio temporal para objects.
        fs::create_dir_all("./.git/objects").expect("Error");

        // Se crea un archivo temporal para el index y se le agrega una entrada.
        let mut index_file = File::create("./.git/index").expect("Error");
        index_file
            .write_all(b"hola.rs blob sj2839sj2k3hs0dlquwjsodm2n3j4djsk3777sja")
            .expect("Error");
        let mut index_file = File::open("./.git/index").expect("Error");
        let mut index_content = String::new();
        index_file
            .read_to_string(&mut index_content)
            .expect("Error");

        // Se chequea que el index se haya creado bien.
        assert_eq!(
            index_content,
            "hola.rs blob sj2839sj2k3hs0dlquwjsodm2n3j4djsk3777sja"
        );

        let result = git_add("./", "testfile");

        // Se chequea que el index se haya modificado correctamente luego de la ejecucion de git_add.
        drop(index_file);
        let mut index_file = File::open("./.git/index").expect("Error");
        let mut index_content = String::new();
        index_file
            .read_to_string(&mut index_content)
            .expect("Error");
        assert_eq!(index_content, "hola.rs blob sj2839sj2k3hs0dlquwjsodm2n3j4djsk3777sja\ntestfile blob e69de29bb2d1d6434b8b29ae775ad8c2e48c5391");

        // Se elimina el directorio temporal.
        fs::remove_dir_all("./.git").expect("Falló al remover el directorio temporal");
        assert!(result.is_ok());
    }
}
