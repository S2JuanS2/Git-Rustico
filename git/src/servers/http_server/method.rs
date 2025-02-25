use crate::servers::errors::ServerError;
use std::fmt;
use std::sync::{mpsc::Sender, Arc, Mutex};

use super::{
    features_pr::{
        create_pull_requests, delete_pull_request, get_pull_request, list_commits,
        list_pull_request, merge_pull_request, modify_pull_request,
    },
    http_body::HttpBody,
    status_code::StatusCode,
};

/// Enumera los posibles métodos HTTP que pueden ser utilizados en una solicitud.
#[derive(Debug, PartialEq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

impl Method {
    /// Crea un método HTTP a partir de su representación en cadena.
    ///
    /// # Argumentos
    ///
    /// * `method` - Una porción de cadena que representa el método HTTP.
    ///
    /// # Retorna
    ///
    /// Retorna un `Result` que contiene el `Method` si la conversión es exitosa,
    /// o un `ServerError::MethodNotAllowed` si el método no es reconocido.
    ///
    pub fn create_method(method: &str) -> Result<Self, ServerError> {
        match method {
            "GET" => Ok(Method::Get),
            "POST" => Ok(Method::Post),
            "PUT" => Ok(Method::Put),
            "DELETE" => Ok(Method::Delete),
            "PATCH" => Ok(Method::Patch),
            _ => Err(ServerError::MethodNotAllowed),
        }
    }

    /// Maneja una solicitud HTTP basada en el método.
    ///
    /// # Argumentos
    ///
    /// * `path` - La ruta del recurso solicitado.
    /// * `http_body` - El cuerpo de la solicitud HTTP.
    /// * `src` - La dirección de origen de la solicitud.
    /// * `tx` - Un canal para enviar respuestas.
    ///
    /// # Retorna
    ///
    /// Retorna un `Result` que contiene el `StatusCode` si la solicitud se maneja con éxito,
    /// o un `ServerError` si ocurre un error.
    ///
    pub fn handle_method(
        &self,
        path: &str,
        http_body: &HttpBody,
        src: &String,
        tx: &Arc<Mutex<Sender<String>>>,
    ) -> Result<StatusCode, ServerError> {
        match self {
            Method::Get => self.handle_get_request(path, src, tx),
            Method::Post => self.handle_post_request(path, http_body, src, tx),
            Method::Put => self.handle_put_request(path, src, tx),
            Method::Patch => self.handle_patch_request(path, http_body, src, tx),
            Method::Delete => self.handle_delete_request(path, src, tx),
        }
    }

    /// Maneja una solicitud HTTP GET.
    ///
    /// # Parámetros
    ///
    /// * `pr` - Una referencia a la estructura `PullRequest`.
    /// * `src` - Una referencia a la cadena que contiene el directorio fuente.
    /// * `tx` - Un puntero compartido y seguro para subprocesos a un transmisor de mensajes.
    ///
    /// # Retornos
    ///
    /// Devuelve un `Result` que contiene el status en caso de éxito o un `ServerError` en caso de fallo.
    ///
    fn handle_get_request(
        &self,
        path: &str,
        src: &String,
        tx: &Arc<Mutex<Sender<String>>>,
    ) -> Result<StatusCode, ServerError> {
        let path_segments: Vec<&str> = segment_path(path);
        match path_segments.as_slice() {
            ["repos", repo_name, "pulls"] => list_pull_request(repo_name, src, tx),
            ["repos", repo_name, "pulls", pull_number] => {
                get_pull_request(repo_name, pull_number, src, tx)
            }
            ["repos", repo_name, "pulls", pull_number, "commits"] => {
                list_commits(repo_name, pull_number, src, tx)
            }
            _ => Ok(StatusCode::ResourceNotFound(
                "The requested path was not found on the server.".to_string(),
            )),
        }
    }

    /// Maneja una solicitud HTTP POST.
    ///
    /// # Parámetros
    ///
    /// * `pr` - Una referencia a la estructura `PullRequest`.
    /// * `src` - Una referencia a la cadena que contiene el directorio fuente.
    /// * `tx` - Un puntero compartido y seguro para subprocesos a un transmisor de mensajes.
    ///
    /// # Retornos
    ///
    /// Devuelve un `Result` que contiene la respuesta en caso de éxito o un `ServerError` en caso de fallo.
    ///
    fn handle_post_request(
        &self,
        path: &str,
        http_body: &HttpBody,
        src: &String,
        tx: &Arc<Mutex<Sender<String>>>,
    ) -> Result<StatusCode, ServerError> {
        let path_segments: Vec<&str> = segment_path(path);
        match path_segments.as_slice() {
            ["repos", repo_name, "pulls"] => {
                let _tx_lock = match tx.lock() {
                    Ok(lock) => lock,
                    Err(_) => return Err(ServerError::BadRequest("Failed lock".to_string())),
                };
                create_pull_requests(http_body, repo_name, src, tx)
            }
            _ => Ok(StatusCode::ResourceNotFound(
                "The requested path was not found on the server.".to_string(),
            )),
        }
    }

    /// Maneja una solicitud HTTP PUT.
    ///
    /// # Parámetros
    ///
    /// * `pr` - Una referencia a la estructura `PullRequest`.
    /// * `src` - Una referencia a la cadena que contiene el directorio fuente.
    /// * `tx` - Un puntero compartido y seguro para subprocesos a un transmisor de mensajes.
    ///
    /// # Retornos
    ///
    /// Devuelve un `Result` que contiene la respuesta en caso de éxito o un `ServerError` en caso de fallo.
    ///
    fn handle_put_request(
        &self,
        path: &str,
        src: &String,
        tx: &Arc<Mutex<Sender<String>>>,
    ) -> Result<StatusCode, ServerError> {
        let path_segments: Vec<&str> = segment_path(path);
        match path_segments.as_slice() {
            ["repos", repo_name, "pulls", pull_number, "merge"] => {
                let _tx_lock = match tx.lock() {
                    Ok(lock) => lock,
                    Err(_) => return Err(ServerError::BadRequest("Failed lock".to_string())),
                };
                merge_pull_request(repo_name, pull_number, src, tx)
            }
            _ => Ok(StatusCode::ResourceNotFound(
                "The requested path was not found on the server.".to_string(),
            )),
        }
    }

    /// Maneja una solicitud HTTP PATCH.
    ///
    /// # Parámetros
    ///
    /// * `pr` - Una referencia a la estructura `PullRequest`.
    /// * `src` - Una referencia a la cadena que contiene el directorio fuente.
    /// * `tx` - Un puntero compartido y seguro para subprocesos a un transmisor de mensajes.
    ///
    /// # Retornos
    ///
    /// Devuelve un `Result` que contiene la respuesta en caso de éxito o un `ServerError` en caso de fallo.
    ///
    fn handle_patch_request(
        &self,
        path: &str,
        http_body: &HttpBody,
        src: &String,
        tx: &Arc<Mutex<Sender<String>>>,
    ) -> Result<StatusCode, ServerError> {
        let path_segments: Vec<&str> = segment_path(path);
        match path_segments.as_slice() {
            ["repos", repo_name, "pulls", pull_number] => {
                let _tx_lock = match tx.lock() {
                    Ok(lock) => lock,
                    Err(_) => return Err(ServerError::BadRequest("Failed lock".to_string())),
                };
                modify_pull_request(http_body, repo_name, pull_number, src, tx)
            }
            _ => Ok(StatusCode::ResourceNotFound(
                "The requested path was not found on the server.".to_string(),
            )),
        }
    }

    fn handle_delete_request(
        &self,
        path: &str,
        src: &String,
        tx: &Arc<Mutex<Sender<String>>>,
    ) -> Result<StatusCode, ServerError> {
        let path_segments: Vec<&str> = segment_path(path);
        match path_segments.as_slice() {
            ["repos", repo_name, "pulls", pull_number] => {
                let _tx_lock = match tx.lock() {
                    Ok(lock) => lock,
                    Err(_) => return Err(ServerError::BadRequest("Failed lock".to_string())),
                };
                delete_pull_request(repo_name, pull_number, src, tx)
            }
            _ => Ok(StatusCode::ResourceNotFound(
                "The requested path was not found on the server.".to_string(),
            )),
        }
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Patch => "PATCH",
        };
        write!(f, "{}", s)
    }
}

/// Segmenta una ruta en partes separadas.
///
/// Esta función toma una ruta de cadena y la divide en segmentos individuales,
/// eliminando el primer carácter si es una barra diagonal `/`.
///
/// # Argumentos
///
/// * `path` - Una referencia a una cadena que representa la ruta que se va a segmentar.
///
/// # Retorna
///
/// Retorna un vector de porciones de cadena (`Vec<&str>`) que representan los segmentos de la ruta.
///
pub fn segment_path(path: &str) -> Vec<&str> {
    // debo eliminar el 1ero si es solo un /
    let mut path = path;
    if path.starts_with('/') {
        path = &path[1..];
    }
    path.split('/').collect()
}
