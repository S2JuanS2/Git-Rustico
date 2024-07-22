use std::fmt;

use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use crate::servers::errors::ServerError;

#[derive(Debug, PartialEq)]
pub enum HttpBody {
    Json(JsonValue),
    Xml(JsonValue),
    Yaml(YamlValue),
    PlainText(String),
}



impl fmt::Display for HttpBody {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HttpBody::Json(json) => write!(f, "{}", json),
            HttpBody::Xml(xml) => write!(f, "{:?}", xml),
            HttpBody::Yaml(yaml) => write!(f, "{:?}", yaml),
            HttpBody::PlainText(plain_text) => write!(f, "{}", plain_text),
        }
    }
}

impl HttpBody {
    pub fn parse(content_type: &str, body: &str) -> Result<Self, ServerError> {
        match content_type {
            "application/json" => {
                serde_json::from_str(body).map(HttpBody::Json).map_err(|_| ServerError::HttpParseJsonBody)
            }
            "application/yaml" | "text/yaml" => {
                serde_yaml::from_str(body).map(HttpBody::Yaml).map_err(|_| ServerError::HttpParseYamlBody)
            }
            "application/xml" | "text/xml" => {
                match serde_xml_rs::from_str::<JsonValue>(body)
                {
                    Ok(xml) => Ok(HttpBody::Xml(xml)),
                    Err(_) => return Err(ServerError::HttpParseXmlBody),
                }
            }
            "text/plain" => Ok(HttpBody::PlainText(body.to_string())),
            _ => Err(ServerError::UnsupportedMediaType),
        }
    }

    pub fn get_field(&self, field: &str) -> Result<String, ServerError> {
        match self {
            HttpBody::Json(json) => json[field].as_str()
                .ok_or_else(|| ServerError::HttpFieldNotFound(field.to_string()))
                .map(|s| s.to_string()),
            HttpBody::Xml(xml) => {
                if let Some(owner_value) = xml.get(field)
                {
                    let value = owner_value.get("$value").unwrap();
                    match value.as_str()
                    {
                        Some(string) => Ok(string.to_string()),
                        None => Err(ServerError::HttpFieldNotFound(field.to_string())),
                    }
                } else {
                    Err(ServerError::HttpFieldNotFound(field.to_string()))
                }
            }
            HttpBody::Yaml(yaml) => yaml[field].as_str()
                .ok_or_else(|| ServerError::HttpFieldNotFound(field.to_string()))
                .map(|s| s.to_string()),
            HttpBody::PlainText(_) => Err(ServerError::UnsupportedMediaType),
        }
    }
}
