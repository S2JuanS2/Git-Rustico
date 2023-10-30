use crate::consts::*;
use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::formats::hash_generate;
use std::fs::{self, File};
use std::io::Read;
use std::io::Write;

/// Esta función se encarga de llamar al comando rm con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función rm
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_rm(args: Vec<&str>, client: Client) -> Result<(), GitError> {
    if args.len() != 1 {
        return Err(GitError::InvalidArgumentCountRmError);
    }
    let directory = client.get_directory_path();
    let file_name = args[0];
    git_rm(&directory, file_name)
}

/// Remueve un archivo del working directory y del index.
/// ###Parametros
/// 'directory': directorio del repositorio local.
/// 'file_name': nombre del archivo a remover.
pub fn git_rm(directory: &str, file_name: &str) -> Result<(), GitError> {
    match compare_hash(file_name, directory) {
        Ok(can_rm) => can_rm,
        Err(_) => return Err(GitError::ReadFileError),
    };

    Ok(())
}

/// Compara el hash del archivo que se quiere remover con el hash de ese archivo que esta en el index.
/// ###Parametros
/// 'file_name': nombre del archivo a remover.
/// 'directory': directorio del repositorio local.
fn compare_hash(file_name: &str, directory: &str) -> Result<(), GitError> {
    let file_path = format!("{}/{}", directory, file_name);
    let file_content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(_) => return Err(GitError::ReadFileError),
    };

    let header = format!("{} {}\0",BLOB, file_content.len());
    let store = header + &file_content;
    let hash_file = hash_generate(&store);

    match remove_from_index(directory, file_name, hash_file.as_str()) {
        Ok(_) => {}
        Err(_) => return Err(GitError::ReadFileError), // CAMBIAR ERROR
    };

    Ok(())
}

/// Obtiene el hash del archivo que se quiere remover del index y lo compara con el hash del archivo
/// que se quiere remover del working directory. Si son iguales, se remueve del index y del working directory.
/// ###Parametros
/// 'directory': directorio del repositorio local.
/// 'file_name': nombre del archivo a remover.
/// 'hash_file': hash del archivo que se quiere remover del index.
fn remove_from_index(directory: &str, file_name: &str, hash_file: &str) -> Result<(), GitError> {
    // directory/.git/index
    let directory_git = format!("{}/{}", directory, GIT_DIR);
    let index_file_path = format!("{}/{}", directory_git, INDEX);
    let index_file_path = index_file_path.as_str();

    let index_file = File::open(index_file_path);
    let mut index_file = match index_file {
        Ok(file) => file,
        Err(_) => return Err(GitError::OpenFileError),
    };
    let mut index_content: String = String::new();
    let read_index_file = index_file.read_to_string(&mut index_content);
    let _ = match read_index_file {
        Ok(file) => file,
        Err(_) => return Err(GitError::ReadFileError),
    };

    // Divide el contenido del índice en líneas.
    let mut lines: Vec<String> = index_content.lines().map(String::from).collect();

    // Encuentra la línea que empiece con el nombre del archivo.
    let index_hash = lines.iter().position(|line| line.starts_with(file_name));

    // Si se encuentra la línea, elimínala del vector.
    if let Some(index) = index_hash {
        if lines[index].ends_with(hash_file) {
            lines.remove(index);
            let file_path = format!("{}/{}", directory, file_name);
            // Se remueve del working directory
            match fs::remove_file(file_path) {
                Ok(_) => {}
                Err(_) => return Err(GitError::RemoveFileError),
            };
        } else {
            println!("No se puede remover el archivo porque no esta en su version mas reciente.");
        }
    }

    let mut index_file = match File::create(index_file_path) {
        Ok(file) => file,
        Err(_) => return Err(GitError::CreateFileError),
    };

    // Escribe las líneas restantes en el archivo de índice.
    for line in &lines {
        if writeln!(index_file, "{}", line).is_err() {
            return Err(GitError::WriteFileError);
        };
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn rm_test() {
        // Se crea un directorio temporal.
        fs::create_dir_all("a/.git").expect("Error");

        // Se crea un archivo temporal y se escribe algo.
        let mut rm_file = File::create(format!("{}remove.rs", "a/")).expect("Error");
        rm_file.write_all(b"hola").expect("Error");

        // Se crea un archivo temporal para el index y se le agregan dos entradas.
        let mut index_file = File::create("a/.git/index").expect("Error");
        index_file.write_all(b"remove.rs blob b8b4a4e2a5db3ebed5f5e02beb3e2d27bca9fc9a\nhola.rs blob sjdi293usjdkosju29eue2993sjhdia9992udhh0").expect("Error");
        let mut index_file = File::open("a/.git/index").expect("Error");
        let mut index_content = String::new();
        index_file
            .read_to_string(&mut index_content)
            .expect("Error");

        // Se chequea que el index se haya creado bien.
        assert_eq!(index_content, "remove.rs blob b8b4a4e2a5db3ebed5f5e02beb3e2d27bca9fc9a\nhola.rs blob sjdi293usjdkosju29eue2993sjhdia9992udhh0");

        let result = git_rm("a/", "remove.rs");

        // Se chequea que el index se haya modificado correctamente luego de la ejecucion de git_add.
        drop(index_file);
        let mut index_file = File::open("a/.git/index").expect("Error");
        let mut index_content = String::new();
        index_file
            .read_to_string(&mut index_content)
            .expect("Error");
        assert_eq!(
            index_content,
            "hola.rs blob sjdi293usjdkosju29eue2993sjhdia9992udhh0\n"
        );

        fs::remove_dir_all("a/").expect("Falló al remover el directorio temporal");
        assert!(result.is_ok());
    }
}
