use crate::consts::*;
use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::files::*;
use crate::util::formats::{compressor_object, hash_generate};
use chrono::{DateTime, Local};
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::commands::branch::get_current_branch;

const COMMIT_EDITMSG: &str = "COMMIT_EDITMSG";
const BRANCH_DIR: &str = "refs/heads/";

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
/// 'args': Vector de Strings que contiene los parametros que se le pasaran al comando commit
/// 'client': Cliente que contiene el directorio del repositorio local
pub fn handle_commit(args: Vec<&str>, client: Client) -> Result<(), GitError> {
    if args.len() != 2 {
        return Err(GitError::InvalidArgumentCountCommitError);
    }
    if args[0] != "-m" {
        return Err(GitError::FlagCommitNotRecognizedError);
    }
    let directory = client.get_directory_path();

    let message = args[1];

    let test_commit = Commit::new(
        message.to_string(),
        client.get_name().to_string(),
        client.get_email().to_string(),
        client.get_name().to_string(),
        client.get_email().to_string(),
        "01234567".to_string(),
        "ABCDEF1234".to_string(),
    );

    git_commit(directory, test_commit)
}

/// Creará el archivo donde se guarda el mensaje del commit
/// ###Parametros:
/// 'directory': Directorio del git
/// 'msg': mensaje del commit
fn commit_msg_edit(directory: &str, msg: String) -> Result<(), GitError> {
    //Archivo COMMIT_EDITMSG, con el ultimo mensaje del commit
    let commit_msg_path = format!("{}/{}/{}", directory, GIT_DIR, COMMIT_EDITMSG);
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

/// Creará el directorio donde se registran los commits y escribirá el contenido en el
/// archivo con el nombre de la branch actual
/// ###Parametros:
/// 'directory': Directorio del git
fn commit_log(directory: &str, content: &str) -> Result<(), GitError> {
    //Registro de commits (logs/)
    let logs_path = format!("{}/{}/logs/refs/heads", directory, GIT_DIR);
    if !Path::new(&logs_path).exists() {
        match fs::create_dir_all(logs_path.clone()) {
            Ok(_) => (),
            Err(_) => return Err(GitError::CreateDirError),
        };
    }
    //escribir content en el archivo con el nombre de la branch actual
    let current_branch = get_current_branch(directory)?;
    let logs_path = format!("{}/{}", logs_path, current_branch);
    let mut file = match OpenOptions::new().append(true).create(true).open(logs_path) {
        Ok(file) => file,
        Err(_) => return Err(GitError::OpenFileError),
    };
    match file.write_all(content.as_bytes()) {
        Ok(_) => (),
        Err(_) => return Err(GitError::WriteFileError),
    };

    Ok(())
}

/// Creará la carpeta con los 2 primeros digitos del hash del objeto commit, y el archivo con los ultimos 38 de nombre.
/// Luego comprimirá el contenido y lo escribirá en el archivo
/// ###Parametros:
/// 'directory': Directorio del git
/// 'hash_commit': hash del objeto commit previamente generado
fn object_commit_save(directory: &str, hash_commit: String, store: String) -> Result<(), GitError> {
    //Crear el objeto commit
    let object_commit_path = format!("{}/{}/objects/{}", directory, GIT_DIR, &hash_commit[..2]);
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
    let file = match File::create(object_commit_path) {
        Ok(file_object) => file_object,
        Err(_) => return Err(GitError::CreateFileError),
    };
    compressor_object(store, file)?;

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

    //tree <hash-del-arbol> -> contiene las referencias a los archivos y directorios
    //parent <hash-del-padre1> -> contiene el commit anterior
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

    let current_branch = get_current_branch(directory)?;
    let branch_current_path = format!("{}/{}/{}{}", directory, GIT_DIR, BRANCH_DIR, current_branch);
    if current_branch == INITIAL_BRANCH && fs::metadata(&branch_current_path).is_err() {
        create_file(&branch_current_path, &hash_commit)?;
    } else {
        let mut file_current_branch = match OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(branch_current_path)
        {
            Ok(file_current_branch) => file_current_branch,
            Err(_) => return Err(GitError::BranchNotFoundError),
        };
        if file_current_branch
            .write_all(hash_commit.as_bytes())
            .is_err()
        {
            return Err(GitError::WriteFileError);
        }
    }

    object_commit_save(directory, hash_commit, store)?;

    commit_log(directory, &content)?;
    commit_msg_edit(directory, commit.get_message())?;

    //Falta crear el tree con el index
    //Actualizar el index

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::commands::init::git_init;

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

        let directory = "./test_repo";
        git_init(directory).expect("Falló en el comando init");
        let result = git_commit(directory, test_commit);

        fs::remove_dir_all(directory).expect("Falló al remover los directorios");

        assert!(result.is_ok());
    }
}
