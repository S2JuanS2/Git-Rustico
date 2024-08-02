use std::sync::{mpsc::Sender, Arc, Mutex};
use std::collections::HashMap;
use crate::commands::checkout::get_tree_hash;
use crate::commands::merge::merge_pr;
use crate::servers::errors::ServerError;
use crate::util::files::{file_exists, folder_exists, list_directory_contents};
use crate::consts::{APPLICATION_SERVER, FILE, PR_FILE_EXTENSION, PR_FOLDER, PR_MAP_FILE};
use super::pr::{CommitsPr, PullRequest};
use super::pr_registry::{generate_pr_hash_key, pr_already_exists, read_pr_map, update_pr_map};
use super::utils::{get_next_pr_number, save_pr_to_file, setup_pr_directory, valid_repository};
use super::{http_body::HttpBody, status_code::StatusCode};
use crate::commands::branch::get_branch_current_hash;
use crate::commands::cat_file::git_cat_file;
use crate::commands::commit::get_commits;
use crate::commands::push::is_update;



pub fn create_pull_requests(body: &HttpBody, repo_name: &str, src: &String, _tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
    if valid_repository(repo_name, src).is_err() {
        return Ok(StatusCode::ResourceNotFound);
    }
    
    match check_pull_request_changes(repo_name, src, body){
        Ok(_) => {},
        Err(e) => return Ok(e),
    };

    let path = match setup_pr_directory(repo_name, src){
        Ok(p) => p,
        Err(e) => return Ok(e),
    };
    
    let hash_key = match generate_pr_hash_key(body){
        Ok(h) => h,
        Err(e) => return Ok(StatusCode::InternalError(e.to_string())),
    };
    let pr_map_path = format!("{}/{}", path, PR_MAP_FILE);
    
    let mut pr_map = read_pr_map(&pr_map_path)?;
    if pr_already_exists(&pr_map, &hash_key) {
        return Ok(StatusCode::ValidationFailed("Pull request already exists.".to_string()));
    }

    let next_pr = get_next_pr_number(&format!("{}/.next_pr", path))?;
    save_pr_to_file(body, &path, next_pr)?;

    update_pr_map(&mut pr_map, &pr_map_path, hash_key, next_pr)?;

    Ok(StatusCode::Created)
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
pub fn list_pull_request(repo_name: &str, src: &String, _tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
    let directory = format!("{}/{}", src, repo_name);
    let pr_repo_folder_path = format!("{}/{}/{}", src, PR_FOLDER, repo_name);
    if !folder_exists(&pr_repo_folder_path)
    {
        return Ok(StatusCode::ResourceNotFound);
    }
    let prs = list_directory_contents(&pr_repo_folder_path)?;
    if prs.len() <= 1 {
        return Ok(StatusCode::InternalError("No pull request was found".to_string()));
    }
    let mut pr_map: HashMap<u32, HttpBody> = HashMap::new();
    for pr in prs {
        if pr != ".next_pr" && pr != "pr_map.json"{
            let pr_path = format!("{}/{}", pr_repo_folder_path, pr);
            let body = HttpBody::create_from_file(APPLICATION_SERVER, &pr_path)?;
            let num = pr.split('.').next().unwrap_or("").parse::<u32>().unwrap_or(0);
            pr_map.insert(num, body);
        }
    }
    let mut pr_list = vec!();
    let mut keys: Vec<&u32> = pr_map.keys().collect();
    keys.sort();
    for &key in &keys{
        let mut pr = PullRequest::default();
        if let Some(body) = pr_map.get(key){
            pr = PullRequest::from_http_body(body)?;
            let changed_files = get_changed_files_pr(&directory, &body.get_field("base")?, &body.get_field("head")?)?;
            pr.set_changed_files(changed_files);
            let commits = get_commits_pr(&directory, &body.get_field("base")?, &body.get_field("head")?)?;
            pr.set_amount_commits(commits.len());
            pr.set_commits(commits);
        }
        pr_list.push(pr);
    }
    let json_str = serde_json::to_string(&pr_list).unwrap();
    let pr_list_body = HttpBody::parse(APPLICATION_SERVER, &json_str)?;
    Ok(StatusCode::Ok(Some(pr_list_body)))
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
pub fn get_pull_request(repo_name: &str, pull_number: &str, src: &String, _tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
    if valid_repository(repo_name, src).is_err() {
        return Ok(StatusCode::ResourceNotFound);
    }
    
    let file_path: String = get_pull_request_file_path(repo_name, pull_number, src);
    if !file_exists(&file_path)
    {
        return Ok(StatusCode::ResourceNotFound);
    }
    let body = HttpBody::create_from_file(APPLICATION_SERVER, &file_path)?;
    let directory = format!("{}/{}", src, repo_name);
    let mut pr = PullRequest::from_http_body(&body)?;

    // TODO| let _mergeable = logica para obtener el estado de fusion

    let changed_files = get_changed_files_pr(&directory, &body.get_field("base")?, &body.get_field("head")?)?;
    pr.set_changed_files(changed_files);
    let commits = get_commits_pr(&directory, &body.get_field("base")?, &body.get_field("head")?)?;
    pr.set_amount_commits(commits.len());
    pr.set_commits(commits);

    let json_str = serde_json::to_string(&pr).unwrap();
    let pr_list_body = HttpBody::parse(APPLICATION_SERVER, &json_str)?;
    Ok(StatusCode::Ok(Some(pr_list_body)))
}

/// Obtiene los commits de un pull request recibido por parámetro
///
/// Esta función lista los commits de un pull request en dónde dado las branches involucradas
/// base <- head se leeran los hashes de los commits de cada una y se compararán 
/// para enviar los commits correspondientes en un vector.
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
pub fn list_commits(repo_name: &str, pull_number: &str, src: &String, _tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
    let file_path = get_pull_request_file_path(repo_name, pull_number, src);
    if !file_exists(&file_path)
    {
        return Ok(StatusCode::ResourceNotFound);
    }
    let body = HttpBody::create_from_file(APPLICATION_SERVER, &file_path)?;

    let commits = get_body_commits_pr(body, src, repo_name)?;
    if commits.len() == 0{
        return Ok(StatusCode::InternalError("The pull request does not contain new commits.".to_string()));
    }
    let json_str = serde_json::to_string(&commits).unwrap();
    let commit_body = HttpBody::parse(APPLICATION_SERVER, &json_str)?;
    
    Ok(StatusCode::Ok(Some(commit_body)))
}

pub fn merge_pull_request(repo_name: &str, pull_number: &str, src: &String, _tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
    let file_path = get_pull_request_file_path(repo_name, pull_number, src);
    if !file_exists(&file_path)
    {
        return Ok(StatusCode::ResourceNotFound);
    }
    let body = HttpBody::create_from_file(APPLICATION_SERVER, &file_path)?;
    
    if body.get_field("state")? != "open"{
        return Ok(StatusCode::InternalError("This pull request is closed".to_string()));
    }
    let directory = format!("{}/{}", src, repo_name);
    let head = body.get_field("head")?;
    let base = body.get_field("base")?;
    let owner = body.get_field("owner")?;
    let title = body.get_field("title")?;
    
    let mut pr = PullRequest::from_http_body(&body)?;
    let result_merge = merge_pr(&directory, &base, &head, &owner, &title, pull_number, repo_name)?;
    if result_merge.contains("Conflict") {
        pr.change_mergeable("false");
        let updated_body = serde_json::to_string(&pr).unwrap();
        let updated_body_http = HttpBody::parse(APPLICATION_SERVER, &updated_body)?;
        updated_body_http.save_body_to_file(&file_path, &APPLICATION_SERVER.to_string())?;
        return Ok(StatusCode::Conflict);
    }

    pr.change_state("closed");
    pr.change_mergeable("true");
    let updated_body = serde_json::to_string(&pr).unwrap();
    let updated_body_http = HttpBody::parse(APPLICATION_SERVER, &updated_body)?;
    updated_body_http.save_body_to_file(&file_path, &APPLICATION_SERVER.to_string())?;

    Ok(StatusCode::MergeWasSuccessful)
}

pub fn modify_pull_request(body: &HttpBody, repo_name: &str, pull_number: &str, src: &String, _tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
    let file_path = get_pull_request_file_path(repo_name, pull_number, src);
    if !file_exists(&file_path)
    {
        return Ok(StatusCode::ResourceNotFound);
    }
    let _pr = match PullRequest::from_http_body(body)
    {
        Ok(pr) => pr,
        Err(_) => return Ok(StatusCode::BadRequest("The request body does not contain a valid Pull Request.".to_string())),
    };
    // LOGICA PARA MODIFICAR UNA SOLICITUD DE EXTRACCION
    Ok(StatusCode::Forbidden)
}


/// Construye la ruta del archivo para una solicitud de extracción específica en el repositorio.
///
/// Esta función genera una cadena de ruta de archivo para una solicitud de extracción dada 
/// basada en el nombre del repositorio, el número de la solicitud de extracción y el directorio de origen.
/// La ruta construida sigue el formato: `src/PR_FOLDER/repo_name/pull_number/PR_FILE_EXTENSION`.
///
/// # Parámetros
///
/// * `repo_name`: Un slice de cadena que contiene el nombre del repositorio.
/// * `pull_number`: Un slice de cadena que contiene el número de la solicitud de extracción.
/// * `src`: Una referencia a una cadena que contiene la ruta del directorio de origen.
///
/// # Retorna
///
/// Una `String` que contiene la ruta completa al archivo de la solicitud de extracción.
///
/// # Ejemplos
///
/// ```
/// let path = get_pull_request_file_path("repo-name", "42", &"/home/user/repos".to_string());
/// assert_eq!(path, "/home/user/repos/pr_folder/repo-name/42.pr");
/// ```
/// 
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
fn get_body_commits_pr(body: HttpBody, src: &str, repo_name: &str) -> Result<Vec<CommitsPr>, ServerError> {
    let head = body.get_field("head")?;
    let base = body.get_field("base")?;
    let directory = format!("{}/{}", src, repo_name);
    let hash_head = get_branch_current_hash(&directory, head.clone())?.to_string();
    let hash_base = get_branch_current_hash(&directory, base.clone())?.to_string();
    let mut result = vec!();

    let mut count_commits: usize = 0;
    if is_update(&directory, &hash_base, &hash_head, &mut count_commits)?{
        return Ok(result);
    }
    let commits_head = get_commits_pr(&directory, &base, &head)?;
    for commit in commits_head {
        let mut commits_pr = CommitsPr::new();
        let commit_content = git_cat_file(&directory, &commit, "-p")?;
        commits_pr.sha_1 = commit.clone();
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
                }
            }
            if parts.len() >= 2{
                if line.starts_with("tree"){
                    commits_pr.tree_hash = parts[1].to_string();
                }else if line.starts_with("parent"){
                    commits_pr.parent = parts[1].to_string();
                }
            }
            commits_pr.message = line.to_string();
        }
        result.push(commits_pr);
    }
    Ok(result)
}

/// Función que recibe 2 branches, compara sus commits y envía los commits que 
/// no estan contenidos en la branch target en un vector.
/// 
/// # Argumentos
/// 
/// * `directory` - Ruta del repositorio del pull request.
/// * `base` - branch target.
/// * `head` - branch origen.
/// 
/// # Retornos
/// Devuelve `Ok(result)` El vector con los hashes de los commits nuevos.
/// Devuelve `Err( )`
fn get_commits_pr(directory: &str, base: &str, head: &str) -> Result<Vec<String>, ServerError> {
    let commits_head = get_commits(&directory, &head)?;
    let commits_base = get_commits(&directory, &base)?;
    let mut result = vec!();
    for commit in commits_head {
        if !commits_base.contains(&commit){
            result.push(commit);
        }
    }
    Ok(result)
}

/// Función que recorre los sub-tree y almacena los archivos en un hashMap
/// 
/// # Argumentos
/// 
/// * `directory` - Ruta del repositorio del pull request.
/// * `pr_files_map` - hashMap donde se almacenan los archivos del commit.
/// * `tree_hash_head` - hash del arbol actual.
/// * `path` - cadena para completar la ruta del archivo.
/// 
/// # Retornos
/// Devuelve `Ok()` Si no hubo errores.
/// Devuelve `Err( )`
fn recovery_tree_pr(directory: &str, pr_files_map: &mut HashMap<String, String>, tree_hash_head: &str, path: &str) -> Result<(), ServerError>{
    let content_tree_head = git_cat_file(directory, &tree_hash_head, "-p")?;
    for line in content_tree_head.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 3{
            if parts[0] == FILE{
                let path_complete = format!("{}{}", path, parts[1].to_string());
                pr_files_map.insert(parts[2].to_string(), path_complete);
            }else{
                let path_complete = format!("{}{}/", path, parts[1].to_string());
                recovery_tree_pr(directory, pr_files_map, parts[2], &path_complete)?;
            }
        }
    }
    Ok(())
}

/// Función que recibe 2 branches, compara los archivos que fueron modificados en las 
/// diferencias entre sus commits y envía los nombres de los mismos en un vector.
/// 
/// # Argumentos
/// 
/// * `directory` - Ruta del repositorio del pull request.
/// * `base` - branch target.
/// * `head` - branch origen.
/// 
/// # Retornos
/// Devuelve `Ok(result)` El vector con los nombres de los archivos modificados.
/// Devuelve `Err( )`
fn get_changed_files_pr(directory: &str, base: &str, head: &str) -> Result<Vec<String>, ServerError>{
    let mut result = vec!();
    let mut pr_files_map_head: HashMap<String, String> = HashMap::new();
    let mut pr_files_map_base: HashMap<String, String> = HashMap::new();
    let head_current_commit = get_branch_current_hash(&directory, head.to_string())?;
    let base_current_commit = get_branch_current_hash(&directory, base.to_string())?;
    let content_commit_head = git_cat_file(directory, &head_current_commit, "-p")?;
    if let Some(tree_hash_head) = get_tree_hash(&content_commit_head) {
        let mut path = "";
        recovery_tree_pr(directory, &mut pr_files_map_head, tree_hash_head, path)?;
        let content_commit_base = git_cat_file(directory, &base_current_commit, "-p")?;
        let tree_hash_base = get_tree_hash(&content_commit_base).unwrap();
        path = "";
        recovery_tree_pr(directory, &mut pr_files_map_base, tree_hash_base, path)?;
    }

    for file in pr_files_map_head.into_iter(){
        if !pr_files_map_base.contains_key(&file.0) {
            result.push(file.1);
        }
    }
    Ok(result)
}

/// Verifica si un pull request contiene cambios antes de proceder con su creación.
///
/// Esta función se asegura de que el pull request sea válido y contenga cambios entre
/// las ramas especificadas. Si no se detectan cambios, se devuelve un error.
///
/// # Argumentos
///
/// * `repo_name` - El nombre del repositorio donde se desea crear el pull request.
/// * `src` - La ruta del directorio del repositorio en el sistema de archivos.
/// * `body` - El cuerpo de la solicitud HTTP que contiene los datos del pull request.
///
/// # Retornos
///
/// Devuelve `Ok(())` si el pull request es válido y contiene cambios.
/// Devuelve `Err(StatusCode::ValidationFailed)` si no se detectan cambios.
/// Devuelve `Err(StatusCode::InternalError)` si ocurre un error durante la validación.
/// 
pub fn check_pull_request_changes(repo_name: &str, src: &String, body: &HttpBody) -> Result<(), StatusCode> {
    match PullRequest::check_pull_request_validity(repo_name, src, body) {
        Ok(changes) => {
            if !changes {
                return Err(StatusCode::ValidationFailed("The pull request does not contain any changes.".to_string()));
            }
        }
        Err(e) => return Err(StatusCode::InternalError(e.to_string())),
    }
    Ok(())
}
