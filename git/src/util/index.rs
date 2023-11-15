use super::files::{open_file, read_file_string};
use crate::consts::INDEX;
use crate::errors::GitError;

/// Maneja el index del repositorio del cliente, lo abre y devuelve su contenido
///
/// # Argumentos
///
/// * `git_dir`: Contiene la dirección del repositorio.
///
/// # Retorno
///
/// Devuelve un `Result` que contiene un (String) en caso de éxito o un error (CommandsError) en caso de fallo.
///
pub fn open_index(git_dir: &str) -> Result<String, GitError> {
    let path_index = format!("{}/{}", git_dir, INDEX);

    let index_file = open_file(&path_index)?;
    read_file_string(index_file)
}
