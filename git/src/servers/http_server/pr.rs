use crate::consts::{APPLICATION_SERVER, PR_FILE_EXTENSION, PR_FOLDER};
use crate::servers::errors::ServerError;
use crate::util::files::file_exists;
use std::sync::{mpsc::Sender, Arc, Mutex};

use super::{http_body::HttpBody, status_code::StatusCode};

#[derive(Debug, PartialEq)]
pub struct PullRequest {
    pub owner: Option<String>,
    pub repo: Option<String>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub head: Option<String>,
    pub base: Option<String>,
    pub mergeable: Option<String>,
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
        let mergeable = body.get_field("mergeable").ok();
        let body = body.get_field("body").ok();
        
        Ok(PullRequest {
            owner,
            repo,
            title,
            body,
            head,
            base,
            mergeable,
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

    pub fn list_pull_request(&self,_repo_name: &str, _src: &String, _tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
        // LOGICA PARA LISTAR LAS SOLICITUDES DE EXTRACCION
        Ok(StatusCode::Forbidden)
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
        let body = HttpBody::create_from_file(APPLICATION_SERVER, &file_path)?;
        // let mergeable = logica para obtener el estado de fusion
        Ok(StatusCode::Ok(Option::Some(body)))
    }

    pub fn list_commits(&self,repo_name: &str, pull_number: &str, src: &String, _tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
        let file_path = get_pull_request_file_path(repo_name, pull_number, src);
        if !file_exists(&file_path)
        {
            return Ok(StatusCode::ResourceNotFound);
        }
        // Logicar para listar los commits
        // let _get_commits(src, repo_name, self.head, self.base); //
        // LOGICA PARA LISTAR LOS COMMITS DE UNA SOLICITUD DE EXTRACCION
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