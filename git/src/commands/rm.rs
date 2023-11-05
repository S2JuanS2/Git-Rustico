use crate::consts::*;
use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::files::{read_file_string, open_file};
use crate::util::formats::hash_generate;
use std::fs::{self, File};
use std::io::Write;

/// Esta función se encarga de llamar al comando rm con los parametros necesarios.
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función rm
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_rm(args: Vec<&str>, client: Client) -> Result<String, GitError> {
    if args.len() != 1 {
        return Err(GitError::InvalidArgumentCountRmError);
    }
    let directory = client.get_directory_path();
    let file_name = args[0];
    git_rm(directory, file_name)
}

/// Remueve un archivo del working directory y del index.
/// ###Parametros
/// 'directory': directorio del repositorio local.
/// 'file_name': nombre del archivo a remover.
pub fn git_rm(directory: &str, file_name: &str) -> Result<String, GitError> {
    let result = compare_hash(file_name, directory)?;

    Ok(result)
}

/// Compara el hash del archivo que se quiere remover con el hash de ese archivo que esta en el index.
/// ###Parametros
/// 'file_name': nombre del archivo a remover.
/// 'directory': directorio del repositorio local.
fn compare_hash(file_name: &str, directory: &str) -> Result<String, GitError> {
    let file_path = format!("{}/{}", directory, file_name);
    let file = open_file(&file_path)?;
    let file_content = read_file_string(file)?;

    let header = format!("{} {}\0", BLOB, file_content.len());
    let store = header + &file_content;
    let hash_file = hash_generate(&store);

    let result = remove_from_index(directory, file_name, &hash_file)?;

    Ok(result)
}

/// Obtiene el hash del archivo que se quiere remover del index y lo compara con el hash del archivo
/// que se quiere remover del working directory. Si son iguales, se remueve del index y del working directory.
/// ###Parametros
/// 'directory': directorio del repositorio local.
/// 'file_name': nombre del archivo a remover.
/// 'hash_file': hash del archivo que se quiere remover del index.
fn remove_from_index(directory: &str, file_name: &str, hash_file: &str) -> Result<String, GitError> {
    let directory_git = format!("{}/{}", directory, GIT_DIR);
    let index_file_path = format!("{}/{}", directory_git, INDEX);

    let index_file = open_file(&index_file_path)?;
    let index_content = read_file_string(index_file)?;

    let mut lines: Vec<String> = index_content.lines().map(String::from).collect();
    let index_hash = lines.iter().position(|line| line.starts_with(file_name));

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
            return Ok("No se puede remover el archivo porque no esta en su version mas reciente.".to_string());
        }
    }

    update_index(index_file_path, lines)?;

    Ok("Archivo removido con éxito!".to_string())
}

/// Actualiza el index con las lineas que se le pasan por parametro.
/// ###Parametros
/// 'index_file_path': path del index.
/// 'lines': lineas que se quieren escribir en el index.
fn update_index(index_file_path: String, lines: Vec<String>) -> Result<(), GitError> {
    let mut index_file = match File::create(index_file_path) {
        Ok(file) => file,
        Err(_) => return Err(GitError::CreateFileError),
    };

    for line in &lines {
        if writeln!(index_file, "{}", line).is_err() {
            return Err(GitError::WriteFileError);
        };
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{commands::init::git_init, util::files::{create_file, create_file_replace}};

    #[test]
    fn rm_test() {
        let directory = "./test_rm";
        git_init(directory).expect("Fallo al inicializar el repositorio");

        let file_name = "remove.rs";
        let file_path = format!("{}/{}", directory, file_name);
        create_file(&file_path, "hola").expect("Fallo al crear el archivo");

        let index_path = format!("{}/{}/{}", directory, GIT_DIR, INDEX);
        let index_content = "remove.rs blob b8b4a4e2a5db3ebed5f5e02beb3e2d27bca9fc9a\nhola.rs blob sjdi293usjdkosju29eue2993sjhdia9992udhh0";
        create_file_replace(&index_path, index_content).expect("Fallo al crear el archivo");

        let index_file = open_file(&index_path).expect("Fallo al abrir el archivo");
        let index_content = read_file_string(index_file).expect("Fallo al leer el archivo");
        // Se chequea que el index tiene remove.rs y hola.rs con sus tipos y hashes correpondientes.
        assert_eq!(index_content, "remove.rs blob b8b4a4e2a5db3ebed5f5e02beb3e2d27bca9fc9a\nhola.rs blob sjdi293usjdkosju29eue2993sjhdia9992udhh0");

        let result = git_rm(directory, "remove.rs");

        // Se chequea que el index se haya modificado correctamente luego de la ejecucion de git_rm.
        let index_file = open_file(&index_path).expect("Fallo al abrir el archivo");
        let new_index_content = read_file_string(index_file).expect("Fallo al leer el archivo");
        assert_eq!(new_index_content, "hola.rs blob sjdi293usjdkosju29eue2993sjhdia9992udhh0\n");

        fs::remove_dir_all(directory).expect("Falló al remover el directorio temporal");

        assert!(result.is_ok());
    }
}
