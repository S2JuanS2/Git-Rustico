use super::{http_body::HttpBody, utils::validate_branch_changes};
use crate::consts::{APPLICATION_SERVER, OPEN};
use crate::servers::errors::ServerError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
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

impl CommitsPr {
    pub fn new() -> Self {
        Self {
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

impl Default for CommitsPr {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct PullRequest {
    pub id: Option<usize>,
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
    pub commits: Option<Vec<String>>,
    pub amount_commits: Option<usize>,
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
        let id = body
            .get_field("id")
            .ok()
            .and_then(|s| s.parse::<usize>().ok());
        let owner = body.get_field("owner").ok();
        let repo = body.get_field("repo").ok();
        let title = body.get_field("title").ok();
        let head = body.get_field("head").ok();
        let base = body.get_field("base").ok();
        let state = body.get_field("state").ok();
        let body = body.get_field("body").ok();

        Ok(PullRequest {
            id,
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
            amount_commits: None,
        })
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
        let _state = OPEN.to_string();

        let has_changes = validate_branch_changes(repo_name, base_path, &base, &head)?;

        if let Ok(repo) = http_body.get_field("repo") {
            if repo != repo_name {
                return Err(ServerError::InvalidRequestNoChange(
                    "The repository name does not match the repository name in the URL."
                        .to_string(),
                ));
            }
        }

        Ok(has_changes)
    }

    pub fn change_state(&mut self, new_state: &str) {
        self.state = Some(new_state.to_string());
    }

    pub fn change_mergeable(&mut self, mergeable: &str) {
        self.mergeable = Some(mergeable.to_string());
    }
    pub fn set_changed_files(&mut self, files: Vec<String>) {
        self.changed_files = Some(files);
    }
    pub fn set_amount_commits(&mut self, amount: usize) {
        self.amount_commits = Some(amount);
    }
    pub fn set_commits(&mut self, commits: Vec<String>) {
        self.commits = Some(commits);
    }

    pub fn is_open(&self) -> bool {
        self.state.as_deref() == Some(OPEN)
    }

    pub fn close(&mut self) {
        self.state = Some("closed".to_string());
    }

    pub fn get_base(&self) -> Option<&str> {
        match &self.base {
            Some(base) => Some(base),
            None => None,
        }
    }

    pub fn get_head(&self) -> Option<&str> {
        match &self.head {
            Some(head) => Some(head),
            None => None,
        }
    }

    pub fn set_number(&mut self, number: usize) {
        self.id = Some(number);
    }

    pub fn change_title(&mut self, new_title: &str) {
        self.title = Some(new_title.to_string());
    }

    pub fn change_body(&mut self, new_body: &str) {
        self.body = Some(new_body.to_string());
    }

    pub fn change_base(&mut self, new_base: &str) {
        self.base = Some(new_base.to_string());
    }

    pub fn get_id(&self) -> Option<usize> {
        self.id
    }
    pub fn get_amount_commits(&self) -> usize {
        match &self.amount_commits {
            Some(a_c) => *a_c,
            None => 0,
        }
    }
}
