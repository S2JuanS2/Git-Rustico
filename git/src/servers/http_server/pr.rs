use crate::consts::APPLICATION_SERVER;
use crate::servers::errors::ServerError;
use serde::{Serialize,Deserialize};
use super::{http_body::HttpBody, utils::validate_branch_changes};


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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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

    /// Valida un pull request verificando el cuerpo de la solicitud y los cambios en las ramas.
    ///
    /// Esta función extrae los campos necesarios del cuerpo de la solicitud HTTP para un pull request
    /// y realiza verificaciones para asegurar que:
    /// - Las ramas `head` y `base` sean válidas.
    /// - Existan cambios entre las ramas `head` y `base`.
    /// - El nombre del repositorio en el cuerpo de la solicitud coincida con el nombre del repositorio en la URL.
    ///
    /// # Argumentos
    ///
    /// * `repo_name` - El nombre del repositorio según se especifica en la URL.
    /// * `base_path` - La ruta base del directorio donde se almacenan los repositorios.
    /// * `http_body` - El cuerpo de la solicitud HTTP que contiene los detalles del pull request.
    ///
    /// # Retorna
    ///
    /// * `Result<bool, ServerError>` - Retorna `Ok(true)` si el pull request es válido y existen cambios,
    ///   de lo contrario, retorna un `ServerError` indicando el tipo de fallo de validación.
    ///
    /// # Errores
    ///
    /// * `ServerError::HttpFieldNotFound` - Si faltan campos requeridos como `head`, `base`, `owner`, `title` o `body` en la solicitud.
    /// * `ServerError::InvalidRequestNoChange` - Si el nombre del repositorio en el cuerpo no coincide con el de la URL o si no se encuentran cambios entre las ramas.
    ///
    pub fn check_pull_request_validity(
        repo_name: &str,
        base_path: &str,
        http_body: &HttpBody,
    ) -> Result<bool, ServerError> {
        let head = http_body.get_field("head")?;
        let base = http_body.get_field("base")?;
        let _owner = http_body.get_field("owner")?;
        let _title = http_body.get_field("title")?;
        let _body = http_body.get_field("body")?;
        let _state = "open".to_string();
    
        let has_changes = validate_branch_changes(repo_name, base_path, &base, &head)?;

        if let Ok(repo) = http_body.get_field("repo") {
            if repo != repo_name {
                return Err(ServerError::InvalidRequestNoChange(
                    "The repository name does not match the repository name in the URL.".to_string(),
                ));
            }
        }
    
        Ok(has_changes)
    }
}

