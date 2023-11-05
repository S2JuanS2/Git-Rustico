use crate::consts::*;
use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::files::*;
use crate::util::objects::{builder_object_commit, builder_object_tree};
use chrono::{DateTime, Local};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::commands::branch::get_current_branch;

use super::errors::CommandsError;
use super::status::get_index_content;

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
}

impl Commit {
    pub fn new(
        message: String,
        author_name: String,
        author_email: String,
        committer_name: String,
        committer_email: String,
    ) -> Self {
        let date_time = Local::now();

        Commit {
            message,
            author_name,
            author_email,
            committer_name,
            committer_email,
            date: date_time,
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
}

/// Esta función se encarga de llamar al comando commit con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de Strings que contiene los parametros que se le pasaran al comando commit
/// 'client': Cliente que contiene el directorio del repositorio local
pub fn handle_commit(args: Vec<&str>, client: Client) -> Result<String, GitError> {
    if args.len() != 2 {
        return Err(GitError::InvalidArgumentCountCommitError);
    }
    if args[0] != "-m" {
        return Err(GitError::FlagCommitNotRecognizedError);
    }
    let directory = client.get_directory_path();

    let message = args[1];

    let commit = Commit::new(
        message.to_string(),
        client.get_name().to_string(),
        client.get_email().to_string(),
        client.get_name().to_string(),
        client.get_email().to_string(),
    );

    git_commit(directory, commit)
}

/// Creará el archivo donde se guarda el mensaje del commit
/// ###Parametros:
/// 'directory': Directorio del git
/// 'msg': mensaje del commit
fn builder_commit_msg_edit(directory: &str, msg: String) -> Result<(), GitError> {
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
pub fn builder_commit_log(directory: &str, content: &str, hash_commit: &str) -> Result<(), GitError> {
    let logs_path = format!("{}/{}/logs/refs/heads", directory, GIT_DIR);
    if !Path::new(&logs_path).exists() {
        match fs::create_dir_all(logs_path.clone()) {
            Ok(_) => (),
            Err(_) => return Err(GitError::CreateDirError),
        };
    }
    let mut lines: Vec<&str> = content.lines().collect();
    if let Some(first_line) = lines.first_mut(){
        *first_line = hash_commit;
    }
    let content_mod = lines.join("\n");
    let content_mod_with_newline = format!("\n{}", content_mod);
    let current_branch = get_current_branch(directory)?;
    let logs_path = format!("{}/{}", logs_path, current_branch);
    let mut file = match OpenOptions::new().append(true).create(true).open(logs_path) {
        Ok(file) => file,
        Err(_) => return Err(GitError::OpenFileError),
    };
    match file.write_all(content_mod_with_newline.as_bytes()) {
        Ok(_) => (),
        Err(_) => return Err(GitError::WriteFileError),
    };
    Ok(())
}

/// Funcion que crea el contenido a comprimir del objeto commit
/// tree <hash-del-arbol> -> contiene las referencias a los archivos y directorios
/// author Nombre del Autor <correo@ejemplo.com> Fecha
/// committer Nombre del Commitador <correo@ejemplo.com> Fecha
///
/// ###Parametros:
/// 'commit': Estructura que contiene la información del commit
fn commit_content_format(commit: &Commit, tree_hash: &str, parent_hash: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time error")
        .as_secs();

    let content = format!(
        "tree {}\nparent {}\nauthor {} <{}> {} -0300\ncommitter {} <{}> {} -0300\n\n{}\n",
        tree_hash,
        parent_hash,
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
pub fn git_commit(directory: &str, commit: Commit) -> Result<String, GitError> {
    let git_dir = format!("{}/{}", directory, GIT_DIR);
    check_index_content(&git_dir)?;

    let current_branch = get_current_branch(directory)?;
    let branch_current_path = format!("{}/{}{}", git_dir, BRANCH_DIR, current_branch);
    
    let parent_hash;
    let mut contents = String::new();
    if fs::metadata(&branch_current_path).is_ok(){
        let file = open_file(&branch_current_path)?;
        contents = read_file_string(file)?;
    }
    if contents.is_empty() {
        parent_hash = PARENT_INITIAL.to_string();
    }else{
        parent_hash = contents;
    }; 

    let tree_hash = builder_object_tree(&git_dir)?;
    let content = commit_content_format(&commit, &tree_hash, &parent_hash);
    let hash_commit = builder_object_commit(&content, &git_dir)?;
    builder_commit_log(directory, &content, &hash_commit)?;
    builder_commit_msg_edit(directory, commit.get_message())?;

    create_or_replace_commit_into_branch(current_branch, branch_current_path, hash_commit)?;

    let index_path = format!("{}/index", git_dir);
    create_file_replace(&index_path, "")?;

    Ok("Commit exitoso!".to_string())
}

/// Esta función cambia el hash del commit en el archivo de la branch actual (y si no existe el path de
/// la branch actual lo crea).
/// ###Parametros:
/// 'current_branch': Nombre de la branch actual.
/// 'branch_current_path': Path del archivo de la branch actual.
/// 'hash_commit': Hash del commit a escribir.
fn create_or_replace_commit_into_branch(current_branch: String, branch_current_path: String, hash_commit: String) -> Result<(), GitError> {
    if current_branch == INITIAL_BRANCH && fs::metadata(&branch_current_path).is_err() {
        create_file(&branch_current_path, &hash_commit)?;
    } else {
        create_file_replace(&branch_current_path, &hash_commit)?;
    }
    Ok(())
}

/// Esta función chequea que el index no este vacio.
/// ###Parametros:
/// 'git_dir': Directorio del git
fn check_index_content(git_dir: &String) -> Result<(), GitError> {
    let index_content = get_index_content(git_dir)?;
    if index_content.trim().is_empty() {
        return Err(CommandsError::CommitEmptyIndex.into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read};

    use crate::commands::{add::git_add, init::git_init};

    use super::*;

    #[test]
    fn commit_test() {
        let directory = "./test_commit_repo";
        git_init(directory).expect("Falló en el comando init");

        let file_path = format!("{}/{}", directory, "holamundo.txt");
        let mut file = fs::File::create(&file_path).expect("Falló al crear el archivo");
        file.write_all(b"Hola Mundo")
            .expect("Error al escribir en el archivo");

        let test_commit = Commit::new(
            "prueba".to_string(),
            "Juan".to_string(),
            "jdr@fi.uba.ar".to_string(),
            "Juan".to_string(),
            "jdr@fi.uba.ar".to_string(),
        );

        git_add(directory, "holamundo.txt").expect("Fallo en el comando add");

        let result = git_commit(directory, test_commit);

        fs::remove_dir_all(directory).expect("Falló al remover los directorios");

        assert!(result.is_ok());
    }

    #[test]
    fn commit_erase_from_index_test() {
        let directory = "./test_commit_erase_index_repo";
        git_init(directory).expect("Falló en el comando init");
        
        let test_commit = Commit::new(
            "prueba".to_string(),
            "Juan".to_string(),
            "jdr@fi.uba.ar".to_string(),
            "Juan".to_string(),
            "jdr@fi.uba.ar".to_string(),
        );

        let file_path = format!("{}/{}", directory, "testfile.rs");
        let mut file = fs::File::create(&file_path).expect("Falló al crear el archivo");
        file.write_all(b"Hola Mundo")
            .expect("Error al escribir en el archivo");

        git_add(directory, "testfile.rs").expect("Fallo en el comando add");
        let mut index_file = File::open(format!("{}/.git/index", directory)).expect("Error");
        let mut index_content = String::new();
        index_file
            .read_to_string(&mut index_content)
            .expect("Error");

        // Se chequea que el index tenga al testfile.rs luego del add.
        assert_eq!(
            "testfile.rs blob ade1f58b626e2918ca61cc9c8c3bd7f507fd1044",
            index_content
        );

        let result = git_commit(directory, test_commit);

        let mut index_file = File::open(format!("{}/.git/index", directory)).expect("Error");
        let mut index_content = String::new();
        index_file
            .read_to_string(&mut index_content)
            .expect("Error");

        // Se chequea que el index este vacio luego del commit.
        assert_eq!("", index_content);

        fs::remove_dir_all(directory).expect("Falló al remover los directorios");

        assert!(result.is_ok());
    }
}
