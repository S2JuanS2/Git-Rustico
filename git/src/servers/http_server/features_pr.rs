use std::sync::{mpsc::Sender, Arc, Mutex};
use std::collections::HashMap;
use crate::commands::checkout::get_tree_hash;
use crate::commands::merge::{find_commit_common_ancestor, merge_pr};
use crate::servers::errors::ServerError;
use crate::util::files::{file_exists, folder_exists};
use crate::consts::{APPLICATION_SERVER, FILE, OPEN, PR_FILE_EXTENSION, PR_FOLDER, PR_MAP_FILE};
use super::pr::{CommitsPr, PullRequest};
use super::pr_registry::{delete_pr_map, generate_pr_hash_key, pr_already_exists, read_pr_map, update_pr_map};
use super::utils::{get_next_pr_number, save_pr_to_file, setup_pr_directory, valid_repository};
use super::{http_body::HttpBody, status_code::StatusCode};
use crate::commands::branch::get_branch_current_hash;
use crate::commands::cat_file::git_cat_file;
use crate::commands::commit::get_commits;
use crate::commands::push::is_update;

///
/// 
/// 
/// 
/// 
/// 
pub fn create_pull_requests(body: &HttpBody, repo_name: &str, src: &String, _tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
    if valid_repository(repo_name, src).is_err() {
        return Ok(StatusCode::ResourceNotFound("The repository does not exist.".to_string()));
    }  
    
    let path = match setup_pr_directory(repo_name, src){
        Ok(p) => p,
        Err(e) => return Ok(e),
    };
    
    if validate_existing_pr(&body, &path){
        return Ok(StatusCode::ValidationFailed("The pull request already exists.".to_string()));
    }

    match check_pull_request_changes(repo_name, src, body){
        Ok(_) => {},
        Err(e) => return Ok(e),
    };

    let directory = format!("{}/{}", src, repo_name);
    let next_pr = get_next_pr_number(&format!("{}/.next_pr", path))?;
    let mut pr = PullRequest::from_http_body(&body)?;

    pr.change_state(OPEN);
    add_attributes(&directory, body.clone(), &mut pr, next_pr)?;

    let body = HttpBody::create_from_pr(&pr, APPLICATION_SERVER)?;
    
    match add_pr_in_map(&body, &path, next_pr){
        Ok(_) => {},
        Err(e) => return Ok(e),
        
    };
    save_pr_to_file(&body, &path, next_pr)?;

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
        return Ok(StatusCode::ResourceNotFound("The repository does not exist.".to_string()));
    }

    let pr_map_path = format!("{}/{}", pr_repo_folder_path, PR_MAP_FILE);
    let pr_map = read_pr_map(&pr_map_path)?;

    if pr_map.len() == 0 {
        return Ok(StatusCode::InternalError("No pull request was found".to_string()));
    }
    let mut pr_list = vec!();

    for (_key, value) in &pr_map{
        let pr_path = format!("{}/{}.json",pr_repo_folder_path, value);
        let body = HttpBody::create_from_file(APPLICATION_SERVER, &pr_path)?;
        let mut pr;
        pr = PullRequest::from_http_body(&body)?;
        add_attributes(&directory, body.clone(), &mut pr, *value)?;
        if body.get_field("state")? == OPEN {
            pr_list.push(pr);
        }
    }
    let json_str = match serde_json::to_string(&pr_list) {
        Ok(s) => s,
        Err(_) => {
            return Ok(StatusCode::InternalError("Serialize error JSON".to_string()));
        }
    };
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
        return Ok(StatusCode::ResourceNotFound("The repository does not exist.".to_string()));
    }
    
    let file_path: String = get_pull_request_file_path(repo_name, pull_number, src);
    if !file_exists(&file_path)
    {
        return Ok(StatusCode::ResourceNotFound("The pull request does not exist.".to_string()));
    }
    let body = HttpBody::create_from_file(APPLICATION_SERVER, &file_path)?;
    let directory = format!("{}/{}", src, repo_name);

    let mut pr = PullRequest::from_http_body(&body)?;
    match pull_number.parse::<usize>(){
        Ok(value) => {
            add_attributes(&directory, body, &mut pr, value)?;
        }
        Err(_) => return Ok(StatusCode::InternalError("swap fail".to_string())),
    }
    let json_str = match serde_json::to_string(&pr) {
        Ok(s) => s,
        Err(_) => {
            return Ok(StatusCode::InternalError("Serialize error JSON".to_string()));
        }
    };
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
        return Ok(StatusCode::ResourceNotFound("The pull request does not exist.".to_string()));
    }
    let body = HttpBody::create_from_file(APPLICATION_SERVER, &file_path)?;

    let commits = get_body_commits_pr(body, src, repo_name)?;
    if commits.len() == 0{
        return Ok(StatusCode::InternalError("The pull request does not contain new commits.".to_string()));
    }
    let json_str = match serde_json::to_string(&commits) {
        Ok(s) => s,
        Err(_) => {
            return Ok(StatusCode::InternalError("Serialize error JSON".to_string()));
        }
    };
    let commit_body = HttpBody::parse(APPLICATION_SERVER, &json_str)?;
    
    Ok(StatusCode::Ok(Some(commit_body)))
}

pub fn merge_pull_request(repo_name: &str, pull_number: &str, src: &String, _tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
    let file_path = get_pull_request_file_path(repo_name, pull_number, src);
    if !file_exists(&file_path)
    {
        return Ok(StatusCode::ResourceNotFound("The pull request does not exist.".to_string()));
    }
    let body = HttpBody::create_from_file(APPLICATION_SERVER, &file_path)?;
    
    if body.get_field("state")? != OPEN{
        return Ok(StatusCode::InternalError("This pull request is closed".to_string()));
    }
    let directory = format!("{}/{}", src, repo_name);
    let (head, base, owner, title) = match extract_pr_fields(&body) {
        Ok(fields) => fields,
        Err(e) => return Ok(e),
    };
    if !is_mergeable(&directory, &base, &head)? {
        return Ok(StatusCode::Conflict);
    }
    
    let mut pr = PullRequest::from_http_body(&body)?;
    merge_pr(&directory, &base, &head, &owner, &title, pull_number, repo_name)?;

    pr.change_state("closed");
    if let Err(e) = update_pr_attributes(&directory, body, &mut pr, pull_number) {
        return Ok(e);
    }
    let updated_body = match serde_json::to_string(&pr) {
        Ok(s) => s,
        Err(_) => {
            return Ok(StatusCode::InternalError("Serialize error JSON".to_string()));
        }
    };
    let updated_body_http = HttpBody::parse(APPLICATION_SERVER, &updated_body)?;
    updated_body_http.save_body_to_file(&file_path, &APPLICATION_SERVER.to_string())?;

    if let Err(e) = delete_pr_in_map(&updated_body_http, &format!("{}/{}/{}", src, PR_FOLDER, repo_name)) {
        return Ok(e);
    };

    Ok(StatusCode::MergeWasSuccessful)
}

fn extract_pr_fields(body: &HttpBody) -> Result<(String, String, String, String), StatusCode> {
    let head = body.get_field("head").map_err(|_| StatusCode::InternalError("Missing head field".to_string()))?;
    let base = body.get_field("base").map_err(|_| StatusCode::InternalError("Missing base field".to_string()))?;
    let owner = body.get_field("owner").map_err(|_| StatusCode::InternalError("Missing owner field".to_string()))?;
    let title = body.get_field("title").map_err(|_| StatusCode::InternalError("Missing title field".to_string()))?;
    Ok((head, base, owner, title))
}

fn update_pr_attributes(directory: &str, body: HttpBody, pr: &mut PullRequest, pull_number: &str) -> Result<(), StatusCode> {
    match pull_number.parse::<usize>() {
        Ok(value) => {
            add_attributes(directory, body, pr, value).map_err(|_| StatusCode::InternalError("Failed to add attributes".to_string()))
        }
        Err(_) => Err(StatusCode::InternalError("Invalid pull request number".to_string())),
    }
}

pub fn modify_pull_request(body: &HttpBody, repo_name: &str, pull_number: &str, src: &String, _tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
    let file_path = get_pull_request_file_path(repo_name, pull_number, src);
    if !file_exists(&file_path)
    {
        return Ok(StatusCode::ResourceNotFound("The pull request does not exist.".to_string()));
    }
    let _pr = match PullRequest::from_http_body(body)
    {
        Ok(pr) => pr,
        Err(_) => return Ok(StatusCode::BadRequest("The request body does not contain a valid Pull Request.".to_string())),
    };
    // LOGICA PARA MODIFICAR UNA SOLICITUD DE EXTRACCION
    Ok(StatusCode::Forbidden("Pulcito volvio muajaja.".to_string()))
}

/// Elimina una solicitud de extracción del repositorio.
///
/// Esta función elimina una solicitud de extracción, cerrándola si está abierta y
/// actualizando el archivo de mapa de solicitudes de extracción para reflejar el cambio.
///
/// # Parámetros
/// - `repo_name`: El nombre del repositorio al que pertenece la solicitud de extracción.
/// - `pull_number`: El número de la solicitud de extracción que se desea eliminar.
/// - `src`: La ruta base donde se encuentran los archivos del pull request.
/// - `_tx`: Un canal de transmisión (`Sender<String>`) usado para comunicación con el archivo de log.
///
/// # Retornos
/// - `Ok(StatusCode::Ok(None))`: Si la solicitud de extracción se elimina correctamente.
/// - `Ok(StatusCode::ResourceNotFound)`: Si el repositorio o la solicitud de extracción no existen.
/// - `Err(ServerError)`: Si ocurre un error al leer el archivo de la solicitud de extracción o al actualizar el mapa de solicitudes.
/// 
pub fn delete_pull_request(repo_name: &str, pull_number: &str, src: &String, _tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
    let mut pr = match read_and_validate_pull_request(repo_name, pull_number, src) {
        Ok(pr) => pr,
        Err(e) => return Ok(e),
    };
    
    pr.close();

    let body = HttpBody::create_from_pr(&pr, APPLICATION_SERVER)?;
    
    let path = format!("{}/{}/{}", src, PR_FOLDER, repo_name);
    match delete_pr_in_map(&body, &path) {
        Ok(_) => {},
        Err(e) => return Ok(e),
    };
    let file_path = get_pull_request_file_path(repo_name, pull_number, src);
    body.save_body_to_file(&file_path, &APPLICATION_SERVER)?;

    Ok(StatusCode::Ok(None))
}

/// Agrego los atributos "mergeable", "changed_files" "commits" al cuerpo del PullRequest
/// 
/// # Parámetros
/// - `directory`: ruta del repositorio.
/// - `body`: cuerpo del rp.
/// - `pr`: Pull Request a agregar los atributos.
/// 
/// # Retornos
/// - `Err(ServerError)`: Si ocurre un error al leer el archivo de la solicitud de extracción o al actualizar el mapa de solicitudes.
fn add_attributes(directory: &str, body: HttpBody, pr: &mut PullRequest, pull_number: usize) -> Result<(), ServerError>{

    let mergeable = is_mergeable(directory, &body.get_field("base")?, &body.get_field("head")?)?;
    pr.change_mergeable(&mergeable.to_string());
    let changed_files = get_changed_files_pr(directory, &body.get_field("base")?, &body.get_field("head")?)?;
    pr.set_changed_files(changed_files);
    let commits = get_commits_pr(directory, &body.get_field("base")?, &body.get_field("head")?)?;
    pr.set_amount_commits(commits.len());
    pr.set_commits(commits);

    pr.set_number(pull_number);

    Ok(())
} 

/// Agrega una solicitud de extracción al mapa de solicitudes.
///
/// Esta función genera una clave hash para el cuerpo del pull request y la usa para
/// verificar si ya existe en el mapa. Si no existe, guarda la solicitud y actualiza el mapa.
///
/// # Parámetros
/// - `body`: El cuerpo HTTP que contiene la información de la solicitud de extracción.
/// - `path`: La ruta base donde se guardan los archivos del pull request.
/// - `next_pr`: El próximo número de solicitud de extracción a asignar.
///
/// # Retornos
/// - `Ok(())`: Si la solicitud de extracción se guarda y se actualiza el mapa correctamente.
/// - `Err(StatusCode::InternalError)`: Si ocurre un error al generar la clave hash o al guardar los archivos.
/// - `Err(StatusCode::ValidationFailed)`: Si la solicitud de extracción ya existe en el mapa.
/// 
fn add_pr_in_map(body: &HttpBody, path: &str, next_pr: usize) -> Result<(), StatusCode> {
    let hash_key = match generate_pr_hash_key(body){
        Ok(h) => h,
        Err(e) => return Err(StatusCode::InternalError(e.to_string())),
    };
    let pr_map_path = format!("{}/{}", path, PR_MAP_FILE);
    
    let mut pr_map = read_pr_map(&pr_map_path)?;
    if pr_already_exists(&pr_map, &hash_key) {
        return Err(StatusCode::ValidationFailed("El pr ya existe.".to_string()));
    }
    
    match save_pr_to_file(body, &path, next_pr){
        Ok(_) => {},
        Err(e) => return Err(StatusCode::InternalError(e.to_string())),
    };

    match update_pr_map(&mut pr_map, &pr_map_path, hash_key, next_pr){
        Ok(_) => {},
        Err(e) => return Err(StatusCode::InternalError(e.to_string())),
    };
    Ok(())
}

fn validate_existing_pr(body: &HttpBody, path: &str) -> bool {
    let hash_key = match generate_pr_hash_key(body){
        Ok(h) => h,
        Err(_) => return false,
    };
    let pr_map_path = format!("{}/{}", path, PR_MAP_FILE);
    let pr_map = match read_pr_map(&pr_map_path){
        Ok(p) => p,
        Err(_) => return false,
    };
    pr_already_exists(&pr_map, &hash_key)
}

/// Elimina una solicitud de extracción del mapa de solicitudes.
///
/// Esta función genera una clave hash para el cuerpo del pull request y la usa para
/// verificar si existe en el mapa. Si existe, elimina la entrada correspondiente.
///
/// # Parámetros
/// - `body`: El cuerpo HTTP que contiene la información de la solicitud de extracción.
/// - `path`: La ruta base donde se encuentra el archivo del mapa de solicitudes.
///
/// # Retornos
/// - `Ok(())`: Si la solicitud de extracción se elimina del mapa correctamente.
/// - `Err(StatusCode::InternalError)`: Si ocurre un error al generar la clave hash o al eliminar la solicitud del mapa.
/// - `Err(StatusCode::ValidationFailed)`: Si la solicitud de extracción no existe en el mapa.
/// 
fn delete_pr_in_map(body: &HttpBody, path: &str) -> Result<(), StatusCode> {
    let hash_key = match generate_pr_hash_key(&body) {
        Ok(h) => h,
        Err(e) => return Err(StatusCode::InternalError(e.to_string())),
    };
    
    let pr_map_path = format!("{}/{}", path, PR_MAP_FILE);
    
    let mut pr_map = read_pr_map(&pr_map_path)?;
    if !pr_already_exists(&pr_map, &hash_key) {
        return Err(StatusCode::ValidationFailed("El pr no existe.".to_string()));
    }
    
    match delete_pr_map(&mut pr_map, &pr_map_path, &hash_key){
        Ok(_) => Ok(()),
        Err(e) => return Err(StatusCode::InternalError(e.to_string())),
    }
}

/// Lee y valida una solicitud de extracción.
///
/// Esta función verifica la existencia del repositorio y del archivo de solicitud de extracción.
/// También valida que la solicitud esté abierta.
///
/// # Parámetros
/// - `repo_name`: El nombre del repositorio al que pertenece la solicitud de extracción.
/// - `pull_number`: El número de la solicitud de extracción que se desea leer.
/// - `src`: La ruta base donde se encuentran los archivos del pull request.
///
/// # Retornos
/// - `Ok(PullRequest)`: Si la solicitud de extracción se lee y valida correctamente.
/// - `Err(StatusCode::ResourceNotFound)`: Si el repositorio o el archivo de la solicitud no existen.
/// - `Err(StatusCode::InternalError)`: Si ocurre un error al leer el archivo de la solicitud.
/// - `Err(StatusCode::Forbidden)`: Si la solicitud de extracción está cerrada y no puede ser eliminada.
/// 
pub fn read_and_validate_pull_request(repo_name: &str, pull_number: &str, src: &String) -> Result<PullRequest, StatusCode> {
    if valid_repository(repo_name, src).is_err() {
        return Err(StatusCode::ResourceNotFound(
            "El repositorio no existe.".to_string(),
        ));
    };

    // Construir la ruta del archivo de la solicitud de extracción
    let file_path = get_pull_request_file_path(repo_name, pull_number, src);
    if !file_exists(&file_path) {
        return Err(StatusCode::ResourceNotFound(
            "La solicitud de extracción no existe.".to_string(),
        ));
    };

    let pr =  match PullRequest::create_from_file(&file_path){
        Ok(pr) => pr,
        Err(_) => return Err(StatusCode::InternalError("Error al leer la solicitud de extracción en la base de datos".to_string())),
    };

    // Valido el status de la solicitud de extracción
    if !pr.is_open() {
        return Err(StatusCode::Forbidden(
            "No se puede eliminar una solicitud de extracción cerrada.".to_string(),
        ));
    };
    Ok(pr)
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
        if let Some (tree_hash_base) = get_tree_hash(&content_commit_base){
            path = "";
            recovery_tree_pr(directory, &mut pr_files_map_base, tree_hash_base, path)?;
        }
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

pub fn is_mergeable(directory: &str, base: &str, head: &str) -> Result<bool, ServerError> {
    let base_current_commit = get_branch_current_hash(&directory, base.to_string())?;
    let head_current_commit = get_branch_current_hash(&directory, head.to_string())?;
    let common_ancestor = find_commit_common_ancestor(directory, &base, &head)?;
    if common_ancestor == base_current_commit {
        return Ok(true);
    }
    let mut pr_files_map_head: HashMap<String, String> = HashMap::new();
    let mut pr_files_map_base: HashMap<String, String> = HashMap::new();
    let content_commit_head = git_cat_file(directory, &head_current_commit, "-p")?;
    if let Some(tree_hash_head) = get_tree_hash(&content_commit_head) {
        let mut path = "";
        recovery_tree_pr(directory, &mut pr_files_map_head, tree_hash_head, path)?;
        let content_commit_base = git_cat_file(directory, &base_current_commit, "-p")?;
        if let Some(tree_hash_base) = get_tree_hash(&content_commit_base){
            path = "";
            recovery_tree_pr(directory, &mut pr_files_map_base, tree_hash_base, path)?;
        }
    }
    for file in pr_files_map_head.into_iter(){
        if !pr_files_map_base.contains_key(&file.0) {
            let file_head = file.1.as_str();
            for (key, value) in &pr_files_map_base {
                if value == file_head {
                    let content_head = git_cat_file(directory, file.0.as_str(), "-p")?;
                    let hash_base = key.as_str();
                    let content_base = git_cat_file(directory, hash_base, "-p")?;
                    if content_head != content_base {
                        return Ok(false);
                    }
                }
            }
        }
    }
    Ok(true)
}