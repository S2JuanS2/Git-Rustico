use crate::errors::GitError;
use chrono::{Local, DateTime};
use std::fs;
use std::io::Write;
use std::path::Path;

const COMMIT_EDITMSG: &str = "COMMIT_EDITMSG";

#[derive(Clone)]
struct Commit {

    message: String,
    author: String,
    commiter: String,
    date: DateTime<Local>,
}

impl Commit {
    pub fn new(message: String, author: String, commiter: String) -> Self{
        
        let date_time = Local::now();

        Commit{
            message,
            author,
            commiter,
            date: date_time,
        }
    }

    pub fn get_message(&self) -> String{
        self.message.to_string()
    }

    pub fn get_author(&self) -> String{
        self.author.to_string()
    }

    pub fn get_date(&self) -> DateTime<Local>{
        self.date
    }

    pub fn get_commiter(&self) -> String{
        self.commiter.to_string()
    }
}

/// Esta función genera y crea el objeto commit
/// ###Parametros:
/// 'message': mensaje del commit
/// 'author': Nombre del autor del commit principal
/// 'commiter': Nombre del que realiza el commit
pub fn commit(directory: &str, commit_msg: String, author: String, commiter: String) -> Result<(), GitError> {

    let commit = Commit::new(commit_msg, author, commiter);

    println!("{}/{}/{}",&commit.get_author(),&commit.get_date(), &commit.get_commiter());
    
    //tree <hash-del-arbol>
    //parent <hash-del-padre1>
    //parent <hash-del-padre2>
    //author Nombre del Autor <correo@ejemplo.com> Fecha
    //committer Nombre del Commitador <correo@ejemplo.com> Fecha
    
    //Mensaje del commit


    //Crear los objetos

    //Actualizar las referencias (HEAD) <-- leer la branch
    //Con el hash del objeto commit

    //Actualizar el index

    //Registro de commits (logs/)
    let logs_path = format!("{}logs/refs/heads", directory);
    if !Path::new(&logs_path).exists() {
        match fs::create_dir_all(logs_path){
            Ok(_) => (),
            Err(_) => return Err(GitError::ReadFileError),
        };
    }

    //Archivo COMMIT_EDITMSG, con el ultimo mensaje del commit
    let commit_msg_path = format!("{}{}", directory, COMMIT_EDITMSG);
    let mut file = match fs::File::create(commit_msg_path){
        Ok(file) => file,
        Err(_) => return Err(GitError::OpenFileError),
    };
    match file.write_all(commit.get_message().as_bytes()) {
        Ok(_) => (),
        Err(_) => return Err(GitError::ReadFileError),
    };


    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_test(){

        let directory = "./";
        let file_test_path = format!("{}{}",directory, COMMIT_EDITMSG);

        let result = commit(directory,"prueba2".to_string(), "Juan".to_string(), "Juan".to_string());

        fs::remove_file(file_test_path).expect("Falló al remover el archivo temporal");
        fs::remove_dir_all("./logs").expect("Falló al remover los directorios");

        assert!(result.is_ok());
    }

}