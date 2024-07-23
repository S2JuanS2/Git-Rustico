use crate::consts::HTPP_SIGNATURE;
use crate::servers::errors::ServerError;
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
        let body = body.get_field("body").ok();
        
        Ok(PullRequest {
            owner,
            repo,
            title,
            body,
            head,
            base,
        })
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

    pub fn get_pull_request(&self,repo_name: &str, pull_number: &str, _src: &String, tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
        let message = format!("GET request to path: /repos/{}/pulls/{}", repo_name, pull_number);
        println!("{}", message);
        log_message_with_signature(&tx, HTPP_SIGNATURE, &message);
        Ok(StatusCode::Forbidden)
    }

    pub fn list_commits(&self,repo_name: &str, pull_number: &str, _src: &String, tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
        let message = format!("GET request to path: /repos/{}/pulls/{}/commits", repo_name, pull_number);
        println!("{}", message);
        log_message_with_signature(&tx, HTPP_SIGNATURE, &message);
        Ok(StatusCode::Forbidden)
    }

    pub fn merge_pull_request(&self,repo_name: &str, pull_number: &str, _src: &String, tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
        let message = format!("PUT request to path: /repos/{}/pulls/{}/merge", repo_name, pull_number);
        println!("{}", message);
        log_message_with_signature(&tx, HTPP_SIGNATURE, &message);
        Ok(StatusCode::Forbidden)
    }

    pub fn modify_pull_request(&self,repo_name: &str, pull_number: &str, _src: &String, tx: &Arc<Mutex<Sender<String>>>) -> Result<StatusCode, ServerError> {
        let message = format!("PATCH request to path: /repos/{}/pulls/{}", repo_name, pull_number);
        println!("{}", message);
        log_message_with_signature(&tx, HTPP_SIGNATURE, &message);
        Ok(StatusCode::Forbidden)
    }
}