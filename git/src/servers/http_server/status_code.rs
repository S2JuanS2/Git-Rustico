use std::fmt;
use std::collections::HashMap;

use super::http_body::HttpBody;

/// Selección de un pull request o múltiples pull request
#[derive(Debug, PartialEq)]
pub enum Class {
    Single(Option<HttpBody>),
    Multiple(HashMap<u32, HttpBody>),
}

/// Enumera los posibles códigos de estado HTTP que pueden ser retornados por el servidor.
#[derive(Debug, PartialEq)]
pub enum StatusCode {
    Created,
    Forbidden,
    ValidationFailed,
    Ok(Class),
    NotModified,
    PassTheAppropriateMediaType,
    ResourceNotFound,
    Unacceptable,
    InternalError(String),
    ServiceUnavailable,
    MergeWasSuccessful,
    MethodNotAllowed,
    Conflict,
    BadRequest(String),
    UnsupportedMediaType,
    HttpVersionNotSupported,
}

impl StatusCode {
    /// Convierte el código de estado en su representación de cadena.
    ///
    /// # Returns
    ///
    /// Retorna una cadena que representa el código de estado HTTP.
    ///
    /// # Examples
    ///
    /// ```
    /// use git::servers::http_server::status_code::StatusCode;
    /// let status = StatusCode::Ok(None);
    /// assert_eq!(status.to_string(), "200 OK");
    /// ```
    pub fn to_string(&self) -> String {
        match self {
            StatusCode::Created => "201 Created".to_string(),
            StatusCode::Forbidden => "403 Forbidden".to_string(),
            StatusCode::ValidationFailed => "422 Validation failed, or the endpoint has been spammed.".to_string(),
            StatusCode::Ok(class) => {
                let status = "200 OK";
                match class {
                    Class::Single(_) => status.to_string(),
                    Class::Multiple(_) => status.to_string(),
                }
            }
            StatusCode::NotModified => "304 Not modified".to_string(),
            StatusCode::PassTheAppropriateMediaType => "200 Pass the appropriate media type to fetch diff and patch formats.".to_string(),
            StatusCode::ResourceNotFound => "404 Resource not found".to_string(),
            StatusCode::Unacceptable => "406 Unacceptable".to_string(),
            StatusCode::InternalError(_) => "500 Internal Error".to_string(),
            StatusCode::ServiceUnavailable => "503 Service unavailable".to_string(),
            StatusCode::MergeWasSuccessful => "200 OK if merge was successful".to_string(),
            StatusCode::MethodNotAllowed => "405 Method Not Allowed".to_string(),
            StatusCode::Conflict => "409 Conflict if sha was provided and pull request head did not match".to_string(),
            StatusCode::BadRequest(_) => "400 Bad Request".to_string(),
            StatusCode::UnsupportedMediaType => "415 Unsupported Media Type".to_string(),
            StatusCode::HttpVersionNotSupported => "505 HTTP Version Not Supported".to_string(),
        }
    }
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}