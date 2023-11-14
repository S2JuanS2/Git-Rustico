use super::files::{open_file, read_file_string};
use crate::consts::INDEX;
use crate::errors::GitError;

pub fn open_index(git_dir: &str) -> Result<String, GitError> {
    let path_index = format!("{}/{}", git_dir, INDEX);

    let index_file = open_file(&path_index)?;
    read_file_string(index_file)
}
