use std::fmt;

use crate::servers::errors::ServerError;

use super::model::Model;

/// Enumera los posibles códigos de estado HTTP que pueden ser retornados por el servidor.
#[derive(Debug, PartialEq)]
pub enum StatusCode {
    Created,
    Forbidden(String),
    ValidationFailed(String),
    Ok(Option<Model>),
    NotModified,
    PassTheAppropriateMediaType,
    ResourceNotFound(String),
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

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            StatusCode::Created => "201 Created",
            StatusCode::Forbidden(_) => "403 Forbidden",
            StatusCode::ValidationFailed(_) => "422 Validation failed, or the endpoint has been spammed.",
            StatusCode::Ok(_) => "200 OK",
            StatusCode::NotModified => "304 Not modified",
            StatusCode::PassTheAppropriateMediaType => "200 Pass the appropriate media type to fetch diff and patch formats.",
            StatusCode::ResourceNotFound(_) => "404 Resource not found",
            StatusCode::Unacceptable => "406 Unacceptable",
            StatusCode::InternalError(_) => "500 Internal Error",
            StatusCode::ServiceUnavailable => "503 Service unavailable",
            StatusCode::MergeWasSuccessful => "200 OK if merge was successful",
            StatusCode::MethodNotAllowed => "405 Method Not Allowed",
            StatusCode::Conflict => "409 Conflict if sha was provided and pull request head did not match",
            StatusCode::BadRequest(_) => "400 Bad Request",
            StatusCode::UnsupportedMediaType => "415 Unsupported Media Type",
            StatusCode::HttpVersionNotSupported => "505 HTTP Version Not Supported",
        };
        write!(f, "{}", s)
    }
}

impl From<ServerError> for StatusCode {
    fn from(error: ServerError) -> Self {
        match error {
            ServerError::BadRequest(e) => StatusCode::BadRequest(e),
            ServerError::UnsupportedMediaType => StatusCode::UnsupportedMediaType,
            ServerError::HttpVersionNotSupported => StatusCode::HttpVersionNotSupported,
            ServerError::MethodNotAllowed => StatusCode::MethodNotAllowed,
            ServerError::ResourceNotFound(s) => StatusCode::ResourceNotFound(s),
            ServerError::InvalidGetPathError => StatusCode::ResourceNotFound(error.to_string()),
            ServerError::InvalidPostPathError => StatusCode::ResourceNotFound(error.to_string()),
            ServerError::InvalidPutPathError => StatusCode::ResourceNotFound(error.to_string()),
            ServerError::InvalidPatchPathError => StatusCode::ResourceNotFound(error.to_string()),
            ServerError::MissingRequestLine => StatusCode::BadRequest("Missing request line".to_string()),
            ServerError::IncompleteRequestLine => StatusCode::BadRequest("Incomplete request line".to_string()),
            ServerError::HttpFieldNotFound(e) => StatusCode::BadRequest(format!("Field not found: {}", e)),
            ServerError::EmptyBody => StatusCode::BadRequest("Empty body".to_string()),
            _ => StatusCode::InternalError("Internal server error".to_string()),
        }
    }
}