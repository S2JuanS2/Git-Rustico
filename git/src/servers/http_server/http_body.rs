use std::fmt;

use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use crate::{consts::{APPLICATION_JSON, APPLICATION_XML, APPLICATION_YAML, TEXT_XML, TEXT_YAML}, servers::errors::ServerError};

/// Enum `HttpBody` que representa los diferentes tipos de cuerpos de solicitudes HTTP.
///
/// Este enum puede contener un valor JSON, XML, YAML o texto plano.
///
/// # Variantes
/// - `Json(JsonValue)`: Contiene un valor JSON.
/// - `Xml(JsonValue)`: Contiene un valor XML representado como `JsonValue`.
/// - `Yaml(YamlValue)`: Contiene un valor YAML.
/// 
#[derive(Debug, PartialEq)]
pub enum HttpBody {
    Json(JsonValue),
    Xml(JsonValue),
    Yaml(YamlValue),
    Empty,
}


/// Implementa el trait `fmt::Display` para `HttpBody`.
///
/// Permite que el tipo `HttpBody` sea formateado como una cadena, dependiendo de su variante.
///
/// # Formateo
/// - Para `Json`: Formatea el contenido JSON usando la representación predeterminada.
/// - Para `Xml`: Usa `{:?}` para mostrar la representación de depuración del XML.
/// - Para `Yaml`: Usa `{:?}` para mostrar la representación de depuración del YAML.
/// 
impl fmt::Display for HttpBody {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HttpBody::Json(json) => write!(f, "{}", json),
            HttpBody::Xml(xml) => write!(f, "{:?}", xml),
            HttpBody::Yaml(yaml) => write!(f, "{:?}", yaml),
            HttpBody::Empty => write!(f, ""),
        }
    }
}

impl HttpBody {
    /// Analiza el cuerpo de la solicitud HTTP según el tipo de contenido especificado.
    ///
    /// # Parámetros
    /// - `content_type`: El tipo de contenido de la solicitud, como `application/json`, `application/xml`, etc.
    /// - `body`: El cuerpo de la solicitud como una cadena.
    ///
    /// # Retorno
    /// Retorna un `Result` que contiene el `HttpBody` adecuado en caso de éxito,
    /// o un `ServerError` en caso de error.
    ///
    /// # Errores
    /// - `ServerError::HttpParseJsonBody` si ocurre un error al analizar JSON.
    /// - `ServerError::HttpParseYamlBody` si ocurre un error al analizar YAML.
    /// - `ServerError::HttpParseXmlBody` si ocurre un error al analizar XML.
    /// - `ServerError::UnsupportedMediaType` si el tipo de contenido no es soportado.
    /// 
    pub fn parse(content_type: &str, body: &str) -> Result<Self, ServerError> {
        if body.is_empty() {
            return Ok(HttpBody::Empty);
        }
        match content_type {
            APPLICATION_JSON => {
                serde_json::from_str(body).map(HttpBody::Json).map_err(|_| ServerError::HttpParseJsonBody)
            }
            APPLICATION_YAML | TEXT_YAML => {
                serde_yaml::from_str(body).map(HttpBody::Yaml).map_err(|_| ServerError::HttpParseYamlBody)
            }
            APPLICATION_XML | TEXT_XML => {
                serde_xml_rs::from_str(body).map(HttpBody::Xml).map_err(|_| ServerError::HttpParseXmlBody)
            }
            _ => Err(ServerError::UnsupportedMediaType),
        }
    }

    /// Obtiene el valor de un campo específico dentro del cuerpo de la solicitud.
    ///
    /// # Parámetros
    /// - `field`: El nombre del campo cuyo valor se desea obtener.
    ///
    /// # Retorno
    /// Retorna un `Result` que contiene el valor del campo como una cadena en caso de éxito,
    /// o un `ServerError` en caso de error.
    ///
    /// # Errores
    /// - `ServerError::HttpFieldNotFound` si el campo no se encuentra en el cuerpo de la solicitud.
    /// - `ServerError::UnsupportedMediaType` si el tipo de cuerpo no es soportado para esta operación.
    /// 
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
            HttpBody::Empty => Err(ServerError::HttpFieldNotFound(field.to_string())),
        }
    }

    /// Crea un cuerpo de solicitud HTTP (`HttpBody`) a partir de un archivo.
    ///
    /// Esta función lee el contenido de un archivo especificado y lo parsea en
    /// función del tipo de contenido proporcionado.
    ///
    /// # Parámetros
    /// - `content_type`: El tipo de contenido del archivo (por ejemplo, "application/json").
    /// - `file_path`: La ruta al archivo que se va a leer.
    ///
    /// # Retornos
    /// - `Ok(HttpBody)`: Si el archivo se lee y parsea correctamente.
    /// - `Err(ServerError)`: Si ocurre un error al leer el archivo o al parsearlo.
    ///
    pub fn create_from_file(content_type: &str, file_path: &str) -> Result<Self, ServerError> {
        let content = match std::fs::read_to_string(file_path)
        {
            Ok(content) => content,
            Err(_) => return Err(ServerError::ResourceNotFound(file_path.to_string())),
        };
        HttpBody::parse(content_type, &content)
    }
}
