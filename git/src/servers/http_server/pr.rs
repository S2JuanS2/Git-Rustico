use crate::consts::{HTPP_SIGNATURE, PR_FOLDER};
use crate::servers::errors::ServerError;
use crate::util::files::file_exists;
use crate::util::logger::log_message_with_signature;
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
        let content = match std::fs::read_to_string(file_path)
        {
            Ok(content) => content,
            Err(_) => return Err(ServerError::ResourceNotFound(file_path.to_string())),
        };
        let body = match HttpBody::create_json(&content)
        {
            Ok(body) => body,
            Err(e) => return Err(e),
        };
        PullRequest::from_http_body(&body)
    }

    pub fn create_pull_requests(&self,repo_name: &str, _src: &String,tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
        let message = format!("POST request to path: /repos/{}/pulls", repo_name);
        println!("{}", message);
        log_message_with_signature(&tx, HTPP_SIGNATURE, &message);
        Ok(StatusCode::Forbidden)
    }

    pub fn list_pull_request(&self,repo_name: &str, _src: &String, tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
        let message = format!("GET request to path: /repos/{}/pulls", repo_name);
        println!("{}", message);
        log_message_with_signature(&tx, HTPP_SIGNATURE, &message);
        Ok(StatusCode::Forbidden)
    }

    pub fn get_pull_request(&self,repo_name: &str, pull_number: &str, src: &String, tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
        let file_path: String = get_pull_request_file_path(repo_name, pull_number, src);
        println!("{}", file_path);
        if !file_exists(&file_path)
        {
            return Ok(StatusCode::ResourceNotFound);
        }
        // let  
        let pr = PullRequest::create_from_file(&file_path);
        println!("{:?}", pr);
        let message = format!("GET request to path: /repos/{}/pulls/{}", repo_name, pull_number);
        println!("{}", message);
        log_message_with_signature(&tx, HTPP_SIGNATURE, &message);
        match pr {
            Ok(_) => Ok(StatusCode::Ok),
            Err(e) => Err(e),
        }
    }

    pub fn list_commits(&self,repo_name: &str, pull_number: &str, src: &String, tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
        let file_path = get_pull_request_file_path(repo_name, pull_number, src);
        if !file_exists(&file_path)
        {
            return Ok(StatusCode::ResourceNotFound);
        }
        
        let message = format!("GET request to path: /repos/{}/pulls/{}/commits", repo_name, pull_number);
        println!("{}", message);
        log_message_with_signature(&tx, HTPP_SIGNATURE, &message);
        Ok(StatusCode::Forbidden)
    }

    pub fn merge_pull_request(&self,repo_name: &str, pull_number: &str, src: &String, tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
        let file_path = get_pull_request_file_path(repo_name, pull_number, src);
        if !file_exists(&file_path)
        {
            return Ok(StatusCode::ResourceNotFound);
        }

        let message = format!("PUT request to path: /repos/{}/pulls/{}/merge", repo_name, pull_number);
        println!("{}", message);
        log_message_with_signature(&tx, HTPP_SIGNATURE, &message);
        Ok(StatusCode::Forbidden)
    }

    pub fn modify_pull_request(&self,repo_name: &str, pull_number: &str, src: &String, tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
        let file_path = get_pull_request_file_path(repo_name, pull_number, src);
        if !file_exists(&file_path)
        {
            return Ok(StatusCode::ResourceNotFound);
        }

        let message = format!("PATCH request to path: /repos/{}/pulls/{}", repo_name, pull_number);
        println!("{}", message);
        log_message_with_signature(&tx, HTPP_SIGNATURE, &message);
        Ok(StatusCode::Forbidden)
    }
}

fn get_pull_request_file_path(repo_name: &str, pull_number: &str, src: &String) -> String {
    format!("{}/{}/{}/{}.json", src, PR_FOLDER, repo_name, pull_number)
}