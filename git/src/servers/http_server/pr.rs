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
        Ok(StatusCode::Forbidden)
    }

    pub fn list_pull_request(&self,_repo_name: &str, _src: &String, _tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
        Ok(StatusCode::Forbidden)
    }

    pub fn get_pull_request(&self,repo_name: &str, pull_number: &str, src: &String, _tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
        let file_path: String = get_pull_request_file_path(repo_name, pull_number, src);
        println!("{}", file_path);
        if !file_exists(&file_path)
        {
            return Ok(StatusCode::ResourceNotFound);
        }
        let body = HttpBody::create_from_file(APPLICATION_SERVER, &file_path)?;
        println!("{:?}", body);
        Ok(StatusCode::Ok(Option::None))
    }

    pub fn list_commits(&self,repo_name: &str, pull_number: &str, src: &String, _tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
        let file_path = get_pull_request_file_path(repo_name, pull_number, src);
        if !file_exists(&file_path)
        {
            return Ok(StatusCode::ResourceNotFound);
        }
        Ok(StatusCode::Forbidden)
    }

    pub fn merge_pull_request(&self,repo_name: &str, pull_number: &str, src: &String, _tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
        let file_path = get_pull_request_file_path(repo_name, pull_number, src);
        if !file_exists(&file_path)
        {
            return Ok(StatusCode::ResourceNotFound);
        }
        Ok(StatusCode::Forbidden)
    }

    pub fn modify_pull_request(&self,repo_name: &str, pull_number: &str, src: &String, _tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
        let file_path = get_pull_request_file_path(repo_name, pull_number, src);
        if !file_exists(&file_path)
        {
            return Ok(StatusCode::ResourceNotFound);
        }
        Ok(StatusCode::Forbidden)
    }
}

fn get_pull_request_file_path(repo_name: &str, pull_number: &str, src: &String) -> String {
    format!("{}/{}/{}/{}{}", src, PR_FOLDER, repo_name, pull_number, PR_FILE_EXTENSION)
}