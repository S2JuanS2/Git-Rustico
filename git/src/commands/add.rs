use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::formats::{compressor_object, hash_generate};
use std::fs;
use std::fs::OpenOptions;
use std::{fs::File, io::Read, io::Write};

/// Esta función se encarga de llamar al comando add con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función add
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_add(args: Vec<&str>, client: Client) -> Result<(), GitError> {
    if args.len() != 1 {
        return Err(GitError::InvalidArgumentCountAddError);
    }
    let directory = client.get_directory_path();
    let file_name = args[0];
    git_add(&directory, file_name)
}

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

    let store = header + String::from_utf8_lossy(&content).as_ref();

    let hash_object = hash_generate(&store);

    let git_dir = format!("{}.git", directory);
    let objects_dir = format!(
        "{}/objects/{}/{}",
        &git_dir,
        &hash_object[..2],
        &hash_object[2..]
    );

    let hash_object_path = format!("{}/objects/{}/", &git_dir, &hash_object[..2]);

    match fs::create_dir_all(hash_object_path) {
        Ok(_) => (),
        Err(_) => return Err(GitError::OpenFileError),
    }

    let file_object = match File::create(objects_dir) {
        Ok(file_object) => file_object,
        Err(_) => return Err(GitError::OpenFileError),
    };

    compressor_object(store, file_object)?;

    // Se actualiza el index.
    if let Some(value) = add_to_index(git_dir, file_name, hash_object) {
        return value;
    }

    Ok(())
}

fn add_to_index(
    git_dir: String,
    file_name: &str,
    hash_object: String,
) -> Option<Result<(), GitError>> {
    let index_path = format!("{}/index", &git_dir);

    // Abre el archivo de índice en modo lectura y escritura.
    let mut index_file = match OpenOptions::new().read(true).write(true).open(&index_path) {
        Ok(index_file) => index_file,
        Err(_) => return Some(Err(GitError::OpenFileError)),
    };

    // Lee el contenido del archivo de índice.
    let mut index_content = String::new();
    if index_file.read_to_string(&mut index_content).is_err() {
        return Some(Err(GitError::ReadFileError));
    }

    // Divide el contenido del índice en líneas.
    let mut lines: Vec<String> = index_content.lines().map(String::from).collect();
    let mut updated = false;

    // Busca si el file_name ya existe en el índice.
    for line in &mut lines {
        if line.starts_with(file_name) {
            // Actualiza la línea existente con el nuevo hash_object.
            *line = format!("{} {} {}", file_name, "blob", hash_object);
            updated = true;
            break;
        }
    }

    // Si el file_name no existía en el índice, agrega una nueva entrada.
    if !updated {
        lines.push(format!("{} {} {}", file_name, "blob", hash_object));
    }

    // Reescribe el contenido del archivo de índice.
    let updated_index_content = lines.join("\n");
    let mut index_file = match File::create(index_path) {
        Ok(index_file) => index_file,
        Err(_) => return Some(Err(GitError::OpenFileError)),
    };
    if index_file.set_len(0).is_err() {
        return Some(Err(GitError::WriteFileError));
    }
    if index_file.write_all(updated_index_content.as_bytes()).is_err() {
        return Some(Err(GitError::WriteFileError));
    }

    None
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
