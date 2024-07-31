use crate::consts::{APPLICATION_SERVER, PR_FILE_EXTENSION, PR_FOLDER};
use crate::servers::errors::ServerError;
use crate::util::files::{file_exists, folder_exists, list_directory_contents};
use std::sync::{mpsc::Sender, Arc, Mutex};
use std::collections::HashMap;

use super::{http_body::HttpBody, status_code::StatusCode, status_code::Class};
use crate::commands::branch::get_branch_current_hash;
use crate::commands::cat_file::git_cat_file;
use crate::commands::commit::get_commits;
use crate::commands::push::is_update;

#[derive(Debug)]
struct CommitsPr {
    sha_1: String,
    tree_hash: String,
    parent: String,
    author_name: String,
    author_email: String,
    committer_name: String,
    committer_email: String,
    message: String,
    date: String,
}

impl CommitsPr{
    pub fn new() -> Self{
        Self{
            sha_1: String::new(),
            tree_hash: String::new(),
            parent: String::new(),
            author_name: String::new(),
            author_email: String::new(),
            committer_name: String::new(),
            committer_email: String::new(),
            message: String::new(),
            date: String::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct PullRequest {
    pub owner: Option<String>,
    pub repo: Option<String>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub head: Option<String>,
    pub base: Option<String>,
    pub state: Option<String>,

    // Campos opcionales, estos no deben estar guardados en el archivo
    // del propio pr, solo se los completa por si se necesitan en algun
    // momento.
    pub mergeable: Option<String>,
    pub changed_files: Option<Vec<String>>,
    pub commits: Option<Vec<usize>>,
}

impl PullRequest {
    /// Crea una nueva instancia de `PullRequest` a partir de un objeto `HttpBody`.
    ///
    /// # Argumentos
    ///
    /// * `body` - Un `HttpBody` que contiene los datos del Pull Request.
    ///
    /// # Errores
    ///
    /// Retorna un `ServerError::HttpFieldNotFound` si no se encuentran los campos requeridos.
    pub fn from_http_body(body: &HttpBody) -> Result<Self, ServerError> {
        let owner = body.get_field("owner").ok();
        let repo = body.get_field("repo").ok();
        let title = body.get_field("title").ok();
        let head = body.get_field("head").ok();
        let base = body.get_field("base").ok();
        let state = body.get_field("state").ok();
        let body = body.get_field("body").ok();
        
        Ok(PullRequest {
            owner,
            repo,
            title,
            body,
            head,
            base,
            state,
            mergeable: None,
            changed_files: None,
            commits: None,
        })
    }

    pub fn default() -> Self {
        PullRequest {
            owner: None,
            repo: None,
            title: None,
            body: None,
            head: None,
            base: None,
            mergeable: None,
            state: None,
            changed_files: None,
            commits: None,
        }
    }

    pub fn create_from_file(file_path: &str) -> Result<Self, ServerError> {
        let body = HttpBody::create_from_file(APPLICATION_SERVER, file_path)?;
        PullRequest::from_http_body(&body)
    }

    pub fn create_pull_requests(&self,_repo_name: &str, _src: &String,_tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
        // LOGICA PARA CREAR UNA SOLICITUD DE EXTRACCION
        Ok(StatusCode::Forbidden)
    }

    /// Obtiene una solicitud de extracción desde el repositorio correspondiente.
    ///
    /// Esta función construye la ruta al repositorio usando el nombre del mismo.
    /// Luego, verifica si el repositorio existe, en caso de que no exista,
    /// devuelve un código de estado `ResourceNotFound`. Se leen los archivos dentro del mismo
    /// se los guarda en un vector y se los inserta en un HashMap en dónde el índice encontrado en el nombre
    /// del archivo es la clave y su contenido parseado es el valor
    ///
    /// # Parámetros
    /// - `repo_name`: El nombre del repositorio al que pertenece el pull request.
    /// - `src`: La ruta base donde se encuentran los archivos del pull request.
    /// - `_tx`: Un canal de transmisión (`Sender<String>`) usado para comunicación con el archivo de log.
    ///
    /// # Retornos
    /// - `Ok(StatusCode::Ok)`: Si el repositorio se encuentra y se listan los pr correctamente.
    /// - `Ok(StatusCode::ResourceNotFound)`: Si el repositorio no existe en el sistema.
    /// - `Err(ServerError)`: Si ocurre un error al crear el cuerpo HTTP desde el archivo.
    ///
    pub fn list_pull_request(&self,repo_name: &str, src: &String, _tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {

        let pr_repo_folder_path = format!("{}/{}/{}", src, PR_FOLDER, repo_name);
        if !folder_exists(&pr_repo_folder_path)
        {
            return Ok(StatusCode::ResourceNotFound);
        }
        let prs = list_directory_contents(&pr_repo_folder_path)?;

        let mut pr_map: HashMap<u32, HttpBody> = HashMap::new();
        for pr in prs {
            let pr_path = format!("{}/{}", pr_repo_folder_path, pr);
            let body = HttpBody::create_from_file(APPLICATION_SERVER, &pr_path)?;
            let num = pr.split('.').next().unwrap_or("").parse::<u32>().unwrap_or(0);
            pr_map.insert(num, body);
        }
        let mut result = String::new();
        let mut keys: Vec<&u32> = pr_map.keys().collect();
        keys.sort();
        for &key in &keys{
            if let Some(body) = pr_map.get(key){
                let format_list_pr = format!("#{} {} {}:{} -> {} [{}]\n", 
                key, 
                body.get_field("title")?,
                body.get_field("owner")?,
                body.get_field("head")?,
                body.get_field("base")?,
                body.get_field("mergeable")?
                );
                result = result + format_list_pr.as_str(); 
            }
        }
        println!("{}", result); //<- enviar esto en un struct serializable
        Ok(StatusCode::Ok(Class::Multiple(pr_map)))
    }

    /// Obtiene una solicitud de extracción desde el archivo correspondiente.
    ///
    /// Esta función construye la ruta al archivo del pull request usando el nombre del repositorio
    /// y el número del pull request. Luego, intenta leer y parsear el archivo. Si el archivo no existe,
    /// devuelve un código de estado `ResourceNotFound`. Si el archivo se lee y parsea correctamente,
    /// devuelve un código de estado `Ok`.
    ///
    /// # Parámetros
    /// - `repo_name`: El nombre del repositorio al que pertenece el pull request.
    /// - `pull_number`: El número del pull request que se desea obtener.
    /// - `src`: La ruta base donde se encuentran los archivos del pull request.
    /// - `_tx`: Un canal de transmisión (`Sender<String>`) usado para comunicación con el archivo de log.
    ///
    /// # Retornos
    /// - `Ok(StatusCode::Ok)`: Si el archivo se encuentra y se parsea correctamente.
    /// - `Ok(StatusCode::ResourceNotFound)`: Si el archivo no existe en el sistema.
    /// - `Err(ServerError)`: Si ocurre un error al crear el cuerpo HTTP desde el archivo.
    ///
    pub fn get_pull_request(&self,repo_name: &str, pull_number: &str, src: &String, _tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
        let file_path: String = get_pull_request_file_path(repo_name, pull_number, src);
        if !file_exists(&file_path)
        {
            return Ok(StatusCode::ResourceNotFound);
        }
        // TODO| let _mergeable = logica para obtener el estado de fusion
        // TODO| let _changed_files = logica para obtener los archivos modificados
        // TODO| let _commits = logica para obtener los commits <- get_commits_pr
        // actualizar el pr con los campos obtenidos
        let body = HttpBody::create_from_file(APPLICATION_SERVER, &file_path)?;
        Ok(StatusCode::Ok(Class::Single(Some(body))))
    }

    pub fn list_commits(&self,repo_name: &str, pull_number: &str, src: &String, _tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
        let file_path = get_pull_request_file_path(repo_name, pull_number, src);
        if !file_exists(&file_path)
        {
            return Ok(StatusCode::ResourceNotFound);
        }
        let body = HttpBody::create_from_file(APPLICATION_SERVER, &file_path)?;

        let commits = get_commits_pr(body, src, repo_name)?;
        println!("{:?}", commits); // <-- Serializar y enviar el cuerpo --
        
        Ok(StatusCode::Forbidden)
    }

    pub fn merge_pull_request(&self,repo_name: &str, pull_number: &str, src: &String, _tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
        let file_path = get_pull_request_file_path(repo_name, pull_number, src);
        if !file_exists(&file_path)
        {
            return Ok(StatusCode::ResourceNotFound);
        }
        // LOGICA PARA FUSIONAR UNA SOLICITUD DE EXTRACCION
        Ok(StatusCode::Forbidden)
    }

    pub fn modify_pull_request(&self,repo_name: &str, pull_number: &str, src: &String, _tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
        let file_path = get_pull_request_file_path(repo_name, pull_number, src);
        if !file_exists(&file_path)
        {
            return Ok(StatusCode::ResourceNotFound);
        }
        // LOGICA PARA MODIFICAR UNA SOLICITUD DE EXTRACCION
        Ok(StatusCode::Forbidden)
    }
}

fn get_pull_request_file_path(repo_name: &str, pull_number: &str, src: &String) -> String {
    format!("{}/{}/{}/{}{}", src, PR_FOLDER, repo_name, pull_number, PR_FILE_EXTENSION)
}

/// Obtiene los commits dado el cuerpo del pull request
///
/// # Parámetros
/// - `body`: Cuerpo del pr
/// - `repo_name`: El nombre del repositorio al que pertenece el pull request.
/// - `src`: La ruta base donde se encuentran los archivos del pull request.
///
/// # Retornos
/// - `Ok(Vec)`: Si se creo correctamente el formato del commit.
///
fn get_commits_pr(body: HttpBody, src: &str, repo_name: &str) -> Result<Vec<CommitsPr>, ServerError> {
    let head = body.get_field("head")?;
    let base = body.get_field("base")?;
    let directory = format!("{}/{}", src, repo_name);
    let hash_head = get_branch_current_hash(&directory, head.clone())?.to_string();
    let hash_base = get_branch_current_hash(&directory, base)?.to_string();

    let mut count_commits: usize = 0;
    if is_update(&directory, &hash_base, &hash_head, &mut count_commits)?{
        println!("No commits\n");
    }
    let mut result = vec!();
    let commits = get_commits(&directory, &head)?;
    let mut index = 0;
    while index < count_commits {
        let mut commits_pr = CommitsPr::new();
        let commit_content = git_cat_file(&directory, &commits[index], "-p")?;
        commits_pr.sha_1 = commits[index].clone();
        let mut lines_commit = commit_content.lines();
        for line in lines_commit.by_ref() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                if line.starts_with("author") {
                    commits_pr.author_name = parts[1].to_string();
                    commits_pr.author_email = parts[2].to_string();
                    let timestamp: i64 = parts[3].parse().unwrap_or(0);
                    commits_pr.date = chrono::DateTime::from_timestamp(timestamp, 0).unwrap().to_string();
                }else if line.starts_with("committer") {
                    commits_pr.committer_name = parts[1].to_string();
                    commits_pr.committer_email = parts[2].to_string();
            }else if parts.len() == 2{
                }if line.starts_with("tree"){
                    commits_pr.tree_hash = parts[1].to_string();
                }else if line.starts_with("parent"){
                    commits_pr.parent = parts[1].to_string();
                }
            }
            commits_pr.message = line.to_string();
        }
        result.push(commits_pr);
        index += 1;
    }
    Ok(result)
}