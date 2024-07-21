// 201	Created
// 403	Forbidden
// 422	Validation failed, or the endpoint has been spammed.
// 200	OK
// 304	Not modified
// 422	Validation failed, or the endpoint has been spammed.
// 200	Pass the appropriate media type to fetch diff and patch formats.
// 304	Not modified
// 404	Resource not found
// 406	Unacceptable
// 500	Internal Error
// 503	Service unavailable
// 200	OK
// 200	if merge was successful
// 403	Forbidden
// 404	Resource not found
// 405	Method Not Allowed if merge cannot be performed
// 409	Conflict if sha was provided and pull request head did not match
// 422	Validation failed, or the endpoint has been spammed.
// 200	OK
// 403	Forbidden
// 422	Validation failed, or the endpoint has been spammed.

use std::io::Write;

use crate::{consts::{CRLF, HTTP_VERSION}, servers::errors::ServerError};


pub enum StatusCode {
    Created,
    Forbidden,
    ValidationFailed,
    Ok,
    NotModified,
    PassTheAppropriateMediaType,
    ResourceNotFound,
    Unacceptable,
    InternalError,
    ServiceUnavailable,
    MergeWasSuccessful,
    MethodNotAllowed,
    Conflict,
}

impl StatusCode {
    pub fn to_string(&self) -> String {
        match self {
            StatusCode::Created => "201 Created".to_string(),
            StatusCode::Forbidden => "403 Forbidden".to_string(),
            StatusCode::ValidationFailed => "422 Validation failed, or the endpoint has been spammed.".to_string(),
            StatusCode::Ok => "200 OK".to_string(),
            StatusCode::NotModified => "304 Not modified".to_string(),
            StatusCode::PassTheAppropriateMediaType => "200 Pass the appropriate media type to fetch diff and patch formats.".to_string(),
            StatusCode::ResourceNotFound => "404 Resource not found".to_string(),
            StatusCode::Unacceptable => "406 Unacceptable".to_string(),
            StatusCode::InternalError => "500 Internal Error".to_string(),
            StatusCode::ServiceUnavailable => "503 Service unavailable".to_string(),
            StatusCode::MergeWasSuccessful => "200 OK if merge was successful".to_string(),
            StatusCode::MethodNotAllowed => "405 Method Not Allowed if merge cannot be performed".to_string(),
            StatusCode::Conflict => "409 Conflict if sha was provided and pull request head did not match".to_string(),
        }
    }

    pub fn send_response_http(&self, writer: &mut dyn Write) -> Result<(), ServerError>
    {
        let response = format!("{} {}{}", HTTP_VERSION, self.to_string(), CRLF);
        match writer.write(response.as_bytes())
        {
            Ok(_) => Ok(()),
            Err(_) => Err(ServerError::SendResponse(response)),
        }
    }
}

