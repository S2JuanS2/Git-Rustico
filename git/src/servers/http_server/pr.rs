use crate::consts::APPLICATION_SERVER;
use crate::servers::errors::ServerError;
use serde::{Serialize,Deserialize};
use super::http_body::HttpBody;

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitsPr {
    pub sha_1: String,
    pub tree_hash: String,
    pub parent: String,
    pub author_name: String,
    pub author_email: String,
    pub committer_name: String,
    pub committer_email: String,
    pub message: String,
    pub date: String,
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

}

