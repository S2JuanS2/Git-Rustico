use crate::consts::*;
use super::errors::CommandsError;
use super::log::insert_line_between_lines;
use crate::models::client::Client;
use crate::util::files::*;
use crate::util::index::{open_index, recovery_index};
use crate::util::objects::builder_object_commit;
use chrono::{DateTime, Local, FixedOffset, Utc};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use crate::commands::branch::get_current_branch;

use super::status::{get_index_content, is_files_to_commit};

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
pub fn handle_commit(args: Vec<&str>, client: Client) -> Result<String, CommandsError> {
    if args.len() != 2 {
        return Err(CommandsError::InvalidArgumentCountCommitError);
    }
    if args[0] != "-m" {
        return Err(CommandsError::FlagCommitNotRecognizedError);
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
fn builder_commit_msg_edit(directory: &str, msg: String) -> Result<(), CommandsError> {
    let commit_msg_path = format!("{}/{}/{}", directory, GIT_DIR, COMMIT_EDITMSG);
    let mut file = match fs::File::create(commit_msg_path) {
        Ok(file) => file,
        Err(_) => return Err(CommandsError::CreateFileError),
    };
    match file.write_all(msg.as_bytes()) {
        Ok(_) => (),
        Err(_) => return Err(CommandsError::WriteFileError),
    };

    Ok(())
}

/// Creará el directorio donde se registran los commits y escribirá el contenido en el
/// archivo con el nombre de la branch actual
/// ###Parametros:
/// 'directory': Directorio del git
pub fn builder_commit_log(
    directory: &str,
    content: &str,
    hash_commit: &str,
    current_branch: &str,
    path_log: &str
) -> Result<(), CommandsError> {//logs/refs/heads
    let logs_path = format!("{}/{}/{}", directory, GIT_DIR, path_log);
    if !Path::new(&logs_path).exists() {
        match fs::create_dir_all(logs_path.clone()) {
            Ok(_) => (),
            Err(_) => return Err(CommandsError::CreateDirError),
        };
    }
    let mut lines: Vec<&str> = content.lines().collect();
    if let Some(first_line) = lines.first_mut() {
        *first_line = hash_commit;
    }
    let content_mod = lines.join("\n");
    let content_mod_with_newline = format!("\n{}", content_mod);
    let logs_path = format!("{}/{}", logs_path, current_branch);
    let mut file = match OpenOptions::new().append(true).create(true).open(logs_path) {
        Ok(file) => file,
        Err(_) => return Err(CommandsError::OpenFileError),
    };
    match file.write_all(content_mod_with_newline.as_bytes()) {
        Ok(_) => (),
        Err(_) => return Err(CommandsError::WriteFileError),
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
    
    let date: DateTime<Utc> = Utc::now();
    let timestamp = date.timestamp();
    let offset = FixedOffset::west_opt(3 * 3600).unwrap().to_string();
    let offset_format: String = offset.chars().filter(|&c| c != ':').collect();  
    if parent_hash == PARENT_INITIAL {
        format!(
            "tree {}\nauthor {} <{}> {} {}\ncommitter {} <{}> {} {}\n\n{}\n",
            tree_hash,
            commit.get_author_name(),
            commit.get_author_email(),
            timestamp,
            offset_format,
            commit.get_committer_name(),
            commit.get_committer_email(),
            timestamp,
            offset_format,
            commit.get_message()
        )
    }else{
        format!(
            "tree {}\nparent {}\nauthor {} <{}> {} {}\ncommitter {} <{}> {} {}\n\n{}\n",
            tree_hash,
            parent_hash,
            commit.get_author_name(),
            commit.get_author_email(),
            timestamp,
            offset_format,
            commit.get_committer_name(),
            commit.get_committer_email(),
            timestamp,
            offset_format,
            commit.get_message()
        )
    }
}

/// Esta función genera y crea el objeto commit
/// ###Parametros:
/// 'directory': Directorio del git
/// 'commit': Estructura que contiene la información del commit
pub fn git_commit(directory: &str, commit: Commit) -> Result<String, CommandsError> {
    let git_dir = format!("{}/{}", directory, GIT_DIR);
    check_index_content(&git_dir)?;

    if is_files_to_commit(directory)? {
        return Ok("nothing to commit, working tree clean".to_string())
    }

    let current_branch = get_current_branch(directory)?;
    let branch_current_path = format!("{}/{}{}", git_dir, BRANCH_DIR, current_branch);

    let mut contents = String::new();
    if fs::metadata(&branch_current_path).is_ok() {
        let file = open_file(&branch_current_path)?;
        contents = read_file_string(file)?;
    }
    let parent_hash = if contents.is_empty() {
        PARENT_INITIAL.to_string()
    } else {
        contents
    };

    let index_content = open_index(&git_dir)?;
    let tree_hash = recovery_index(&index_content, &git_dir)?;

    let mut commit_content = commit_content_format(&commit, &tree_hash, &parent_hash);
    let hash_commit = builder_object_commit(&commit_content, &git_dir)?;
    if commit_content.lines().count() == 5{
        commit_content = insert_line_between_lines(&commit_content, 1, PARENT_INITIAL);
    }
    builder_commit_log(directory, &commit_content, &hash_commit, &current_branch, "logs/refs/heads")?;
    builder_commit_msg_edit(directory, commit.get_message())?;

    create_or_replace_commit_into_branch(current_branch.clone(), branch_current_path, hash_commit.clone())?;

    let response = format!("[{} {}] {}", current_branch, &hash_commit.as_str()[..7], commit.get_message());

    Ok(response)
}

/// Esta función cambia el hash del commit en el archivo de la branch actual (y si no existe el path de
/// la branch actual lo crea).
/// ###Parametros:
/// 'current_branch': Nombre de la branch actual.
/// 'branch_current_path': Path del archivo de la branch actual.
/// 'hash_commit': Hash del commit a escribir.
fn create_or_replace_commit_into_branch(
    current_branch: String,
    branch_current_path: String,
    hash_commit: String,
) -> Result<(), CommandsError> {
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
fn check_index_content(git_dir: &str) -> Result<(), CommandsError> {
    let index_content = get_index_content(git_dir)?;
    if index_content.trim().is_empty() {
        return Err(CommandsError::CommitEmptyIndex);
    }
    Ok(())
}

#[cfg(test)]
mod tests {

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
}
