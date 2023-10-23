use crate::errors::GitError;
use crate::util::formats::hash_generate;

use chrono::{DateTime, Local};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const COMMIT_EDITMSG: &str = "COMMIT_EDITMSG";

#[derive(Clone)]
pub struct Commit {
    message: String,
    author_name: String,
    author_email: String,
    committer_name: String,
    committer_email: String,
    date: DateTime<Local>,
    tree_hash: String,
    parent_hash: String,
}

impl Commit {
    pub fn new(
        message: String,
        author_name: String,
        author_email: String,
        committer_name: String,
        committer_email: String,
        tree_hash: String,
        parent_hash: String,
    ) -> Self {
        let date_time = Local::now();

        Commit {
            message,
            author_name,
            author_email,
            committer_name,
            committer_email,
            date: date_time,
            tree_hash,
            parent_hash,
        }
    }

    pub fn get_message(&self) -> String {
        self.message.to_string()
    }

    pub fn get_author_name(&self) -> String {
        self.author_name.to_string()
    }

    pub fn get_author_email(&self) -> String {
        self.author_email.to_string()
    }

    pub fn get_committer_name(&self) -> String {
        self.committer_name.to_string()
    }

    pub fn get_committer_email(&self) -> String {
        self.committer_email.to_string()
    }

    pub fn get_date(&self) -> DateTime<Local> {
        self.date
    }

    pub fn get_tree_hash(&self) -> String {
        self.tree_hash.to_string()
    }

    pub fn get_parent_hash(&self) -> String {
        self.parent_hash.to_string()
    }
}

/// Esta función se encarga de llamar al comando commit con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de Strings que contiene los parametros que se le pasaran al comando
pub fn handle_commit(args: Vec<&str>) -> Result<(), GitError> {

    if args.len() != 2 {
        return Err(GitError::InvalidArgumentCountCommitError);
    }
    if args[1] != "-m" {
        return Err(GitError::FlagCommitNotRecognizedError);
    }
    let directory = match env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return Err(GitError::DirectoryOpenError),
    };

    let message = args[1];

    let test_commit = Commit::new(
        message.to_string(),
        "Juan".to_string(),
        "jdr@fi.uba.ar".to_string(),
        "Juan".to_string(),
        "jdr@fi.uba.ar".to_string(),
        "01234567".to_string(),
        "ABCDEF1234".to_string(),
    );

    git_commit(directory, commit)?;

    Ok(())
}

/// Creará el archivo donde se guarda el mensaje del commit
/// ###Parametros:
/// 'directory': Directorio del git
/// 'msg': mensaje del commit
fn commit_msg_edit(directory: &str, msg: String) -> Result<(), GitError> {
    //Archivo COMMIT_EDITMSG, con el ultimo mensaje del commit
    let commit_msg_path = format!("{}{}", directory, COMMIT_EDITMSG);
    let mut file = match fs::File::create(commit_msg_path) {
        Ok(file) => file,
        Err(_) => return Err(GitError::CreateFileError),
    };
    match file.write_all(msg.as_bytes()) {
        Ok(_) => (),
        Err(_) => return Err(GitError::WriteFileError),
    };

    Ok(())
}

/// Creará el directorio donde se registran los commits
/// ###Parametros:
/// 'directory': Directorio del git
fn commit_log(directory: &str) -> Result<(), GitError> {
    //Registro de commits (logs/)
    let logs_path = format!("{}logs/refs/heads", directory);
    if !Path::new(&logs_path).exists() {
        match fs::create_dir_all(logs_path) {
            Ok(_) => (),
            Err(_) => return Err(GitError::CreateDirError),
        };
    }
    Ok(())
}

/// Creará la carpeta con los 2 primeros digitos del hash del objeto commit, y el archivo con los ultimos 38 de nombre.
/// Luego comprimirá el contenido y lo escribirá en el archivo
/// ###Parametros:
/// 'directory': Directorio del git
/// 'hash_commit': hash del objeto commit previamente generado
fn object_commit_save(directory: &str, hash_commit: String) -> Result<(), GitError> {
    //Crear el objeto commit
    let object_commit_path = format!("{}/.git/objects/{}", directory, &hash_commit[..2]);
    match fs::create_dir_all(object_commit_path) {
        Ok(_) => (),
        Err(_) => return Err(GitError::CreateDirError),
    }

    let object_commit_path = format!(
        "{}/.git/objects/{}/{}",
        directory,
        &hash_commit[..2],
        &hash_commit[2..]
    );
    match File::create(object_commit_path) {
        Ok(_) => (),
        Err(_) => return Err(GitError::CreateFileError),
    }
    //Falta comprimir el store y guardarlo en ese archivo.
    Ok(())
}

/// Creará el formato del objeto commit
/// ###Parametros:
/// 'commit': Estructura que contiene la información del commit
fn commit_content_format(commit: &Commit) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time error")
        .as_secs();

    //tree <hash-del-arbol>
    //parent <hash-del-padre1>
    //parent <hash-del-padre2>
    //author Nombre del Autor <correo@ejemplo.com> Fecha
    //committer Nombre del Commitador <correo@ejemplo.com> Fecha
    let content = format!(
        "tree {}\nparent {}\nauthor {} <{}> {}\ncommitter {} <{}> {}\n\n{}",
        commit.get_tree_hash(),
        commit.get_parent_hash(),
        commit.get_author_name(),
        commit.get_author_email(),
        timestamp,
        commit.get_committer_name(),
        commit.get_committer_email(),
        timestamp,
        commit.get_message()
    );

    content
}
/// Esta función genera y crea el objeto commit
/// ###Parametros:
/// 'directory': Directorio del git
/// 'commit': Estructura que contiene la información del commit
pub fn git_commit(directory: &str, commit: Commit) -> Result<(), GitError> {
    let content = commit_content_format(&commit);

    let content_bytes = content.as_bytes();
    let content_size = content_bytes.len().to_string();
    let header = format!("commit {}\0", content_size);

    let store = header + &content;

    let hash_commit = hash_generate(&store);
    object_commit_save(directory, hash_commit)?;
    commit_log(directory)?;
    commit_msg_edit(directory, commit.get_message())?;

    //Actualizar las referencias (HEAD) <-- leer la branch
    //Con el hash del objeto commit
    //Actualizar el index

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_test() {
        let test_commit = Commit::new(
            "prueba".to_string(),
            "Juan".to_string(),
            "jdr@fi.uba.ar".to_string(),
            "Juan".to_string(),
            "jdr@fi.uba.ar".to_string(),
            "01234567".to_string(),
            "ABCDEF1234".to_string(),
        );

        let directory = "./";
        let file_test_path = format!("{}{}", directory, COMMIT_EDITMSG);

        let result = git_commit(directory, test_commit);

        fs::remove_file(file_test_path).expect("Falló al remover el archivo temporal");
        fs::remove_dir_all("./logs").expect("Falló al remover los directorios");
        fs::remove_dir_all("./.git").expect("Falló al remover el directorio");

        assert!(result.is_ok());
    }
}
