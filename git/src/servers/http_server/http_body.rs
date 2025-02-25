use std::{collections::HashMap, fmt, fs::File, io::Write};

use crate::{
    consts::{APPLICATION_JSON, APPLICATION_XML, APPLICATION_YAML, TEXT_XML, TEXT_YAML},
    servers::errors::ServerError,
};
use serde_json::Value as JsonValue;
use serde_xml_rs::to_string as xml_to_string;
use serde_yaml::Value as YamlValue;

use super::pr::PullRequest;

/// Enum `HttpBody` que representa los diferentes tipos de cuerpos de solicitudes HTTP.
///
/// Este enum puede contener un valor JSON, XML, YAML o texto plano.
///
/// # Variantes
/// - `Json(JsonValue)`: Contiene un valor JSON.
/// - `Xml(JsonValue)`: Contiene un valor XML representado como `JsonValue`.
/// - `Yaml(YamlValue)`: Contiene un valor YAML.
///
#[derive(Debug, PartialEq, Clone)]
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
            APPLICATION_JSON => serde_json::from_str(body)
                .map(HttpBody::Json)
                .map_err(|_| ServerError::HttpParseJsonBody),
            APPLICATION_YAML | TEXT_YAML => serde_yaml::from_str(body)
                .map(HttpBody::Yaml)
                .map_err(|_| ServerError::HttpParseYamlBody),
            APPLICATION_XML | TEXT_XML => serde_xml_rs::from_str(body)
                .map(HttpBody::Xml)
                .map_err(|_| ServerError::HttpParseXmlBody),
            _ => Err(ServerError::UnsupportedMediaType),
        }
    }

    /// Obtiene el valor de un campo específico dentro del cuerpo de la solicitud que se espera que sea un String
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
            HttpBody::Json(json) => json[field]
                .as_str()
                .ok_or_else(|| ServerError::HttpFieldNotFound(field.to_string()))
                .map(|s| s.to_string()),
            HttpBody::Xml(xml) => {
                if let Some(owner_value) = xml.get(field) {
                    let value = owner_value.get("$value").unwrap();
                    match value.as_str() {
                        Some(string) => Ok(string.to_string()),
                        None => Err(ServerError::HttpFieldNotFound(field.to_string())),
                    }
                } else {
                    Err(ServerError::HttpFieldNotFound(field.to_string()))
                }
            }
            HttpBody::Yaml(yaml) => yaml[field]
                .as_str()
                .ok_or_else(|| ServerError::HttpFieldNotFound(field.to_string()))
                .map(|s| s.to_string()),
            HttpBody::Empty => Err(ServerError::HttpFieldNotFound(field.to_string())),
        }
    }

    /// Obtiene el valor de un campo específico dentro del cuerpo de la solicitud que se espera que sea un array.
    ///
    /// # Parámetros
    /// - `field`: El nombre del campo cuyo valor se desea obtener.
    ///
    /// # Retorno
    /// Retorna un `Result` que contiene el valor del campo como un vector de cadenas en caso de éxito,
    /// o un `ServerError` en caso de error.
    ///
    /// # Errores
    /// - `ServerError::HttpFieldNotFound` si el campo no se encuentra en el cuerpo de la solicitud.
    /// - `ServerError::UnsupportedMediaType` si el tipo de cuerpo no es soportado para esta operación.
    ///
    pub fn get_array_field(&self, field: &str) -> Result<Vec<String>, ServerError> {
        match self {
            HttpBody::Json(json) => {
                if let Some(value) = json.get(field) {
                    if let Some(array) = value.as_array() {
                        Ok(array
                            .iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect())
                    } else {
                        Err(ServerError::HttpFieldNotFound(field.to_string()))
                    }
                } else {
                    Err(ServerError::HttpFieldNotFound(field.to_string()))
                }
            }
            HttpBody::Xml(xml) => {
                if let Some(owner_value) = xml.get(field) {
                    if let Some(array) = owner_value.get("$value").and_then(|v| v.as_array()) {
                        Ok(array
                            .iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect())
                    } else {
                        Err(ServerError::HttpFieldNotFound(field.to_string()))
                    }
                } else {
                    Err(ServerError::HttpFieldNotFound(field.to_string()))
                }
            }
            HttpBody::Yaml(yaml) => {
                if let Some(value) = yaml.get(field) {
                    if let Some(array) = value.as_sequence() {
                        Ok(array
                            .iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect())
                    } else {
                        Err(ServerError::HttpFieldNotFound(field.to_string()))
                    }
                } else {
                    Err(ServerError::HttpFieldNotFound(field.to_string()))
                }
            }
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
        let content = match std::fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(_) => return Err(ServerError::ResourceNotFound(file_path.to_string())),
        };
        HttpBody::parse(content_type, &content)
    }

    /// Crea una instancia de `HttpBody` a partir de una solicitud de extracción (`PullRequest`) y un tipo de contenido especificado.
    ///
    /// Esta función toma un objeto `PullRequest` y lo convierte en una instancia de `HttpBody` según el tipo de contenido
    /// indicado. La conversión se realiza en formato JSON, XML o YAML. Si el tipo de contenido no es soportado o si ocurre
    /// un error durante la serialización, se retorna un error adecuado.
    ///
    /// # Argumentos
    ///
    /// * `pr` - Una referencia al objeto `PullRequest` que se desea convertir. Este objeto contiene la información de la
    ///   solicitud de extracción que se debe serializar.
    /// * `content_type` - Una cadena que indica el tipo de contenido deseado para la conversión. Puede ser uno de los siguientes:
    ///   - `APPLICATION_JSON`
    ///   - `APPLICATION_XML`
    ///   - `APPLICATION_YAML`
    ///   - Cualquier otro tipo de contenido se considerará no soportado.
    ///
    pub fn create_from_pr(pr: &PullRequest, content_type: &str) -> Result<Self, ServerError> {
        match content_type {
            APPLICATION_JSON => {
                let json_value = serde_json::to_value(pr)
                    .map_err(|e| ServerError::Serialization(e.to_string()))?;
                Ok(HttpBody::Json(json_value))
            }
            APPLICATION_XML => {
                let json_str = serde_json::to_string(pr)
                    .map_err(|e| ServerError::Serialization(e.to_string()))?;
                let json_value: JsonValue = serde_json::from_str(&json_str)
                    .map_err(|e| ServerError::Serialization(e.to_string()))?;
                let xml_str = serde_xml_rs::to_string(&json_value)
                    .map_err(|e| ServerError::Serialization(e.to_string()))?;
                let xml_value = serde_xml_rs::from_str(&xml_str)
                    .map_err(|e| ServerError::Serialization(e.to_string()))?;
                Ok(HttpBody::Xml(xml_value))
            }
            APPLICATION_YAML => {
                let yaml_value = serde_yaml::to_value(pr)
                    .map_err(|e| ServerError::Serialization(e.to_string()))?;
                Ok(HttpBody::Yaml(yaml_value))
            }
            _ => Err(ServerError::UnsupportedMediaType),
        }
    }

    /// Obtiene el tipo de contenido y el cuerpo como una cadena de texto.
    ///
    /// # Returns
    ///
    /// Retorna una tupla con el tipo de contenido (Content-Type) y el cuerpo como cadena de texto.
    ///
    /// # Errors
    ///
    /// Retorna `ServerError` si hay algún problema al convertir el cuerpo a una cadena de texto.
    // pub fn get_content_type_and_body(&self) -> Result<(String, String), ServerError> {
    //     let test_body = self.convert_body_pr_in_string()?;
    //     println!("test_body: \n{}", test_body);
    //     let content_type_and_body = match self {
    //         HttpBody::Json(json) => (APPLICATION_JSON.to_string(), self.convert_body_pr_in_string()?),
    //         HttpBody::Xml(xml) => {
    //             let map: HashMap<String, JsonValue> = match serde_json::from_value(xml.clone()){
    //                 Ok(map) => map,
    //                 Err(_) => return Err(ServerError::Serialization("Error converting XML to JSON".to_string())),
    //             };
    //             let mut xml_str = String::new();
    //             for (key, value) in map {
    //                 xml_str.push_str(&format!("<{}>{}</{}>", key, value, key));
    //             };
    //             (APPLICATION_XML.to_string(), self.convert_body_pr_in_string()?)
    //         }

    //         HttpBody::Yaml(yaml) => {
    //             let yaml_str = serde_yaml::to_string(yaml).unwrap();
    //             (APPLICATION_YAML.to_string(), self.convert_body_pr_in_string()?)
    //         }
    //         HttpBody::Empty => ("".to_string(), "".to_string())
    //     };
    //     Ok(content_type_and_body)
    // }

    /// Guarda el cuerpo HTTP en un archivo en el formato especificado.
    ///
    /// # Parámetros
    ///
    /// * `file_path` - La ruta del archivo donde se guardará el cuerpo serializado.
    /// * `application` - El formato en el que se debe guardar el cuerpo: `application/json`, `application/xml`, o `application/yaml`.
    ///
    /// # Retornos
    ///
    /// Retorna `Ok(())` si el cuerpo se guarda correctamente en el archivo.
    /// Retorna un `ServerError` si ocurre algún error durante el proceso de creación del archivo o serialización.
    ///
    /// # Errores
    ///
    /// * `ServerError::SavePr` - Si ocurre un error al crear el archivo o al escribir en él.
    /// * `ServerError::Serialization` - Si ocurre un error al serializar el cuerpo en el formato especificado.
    /// * `ServerError::EmptyBody` - Si el cuerpo está vacío y no se puede guardar.
    /// * `ServerError::InvalidFormat` - Si el formato especificado no es compatible.
    ///
    pub fn save_body_to_file(&self, file_path: &str, application: &str) -> Result<(), ServerError> {
        let mut file = match File::create(file_path) {
            Ok(file) => file,
            Err(_) => return Err(ServerError::SavePr),
        };

        // Convertimos el cuerpo actual al formato especificado por `application`
        let serialized = match application {
            APPLICATION_JSON => match self {
                HttpBody::Json(json) => serde_json::to_string_pretty(json)
                    .map_err(|e| ServerError::Serialization(e.to_string()))?,
                HttpBody::Xml(xml) => {
                    // Convertir XML a JSON antes de serializar
                    let xml_string = xml_to_string(xml)
                        .map_err(|e| ServerError::Serialization(e.to_string()))?;
                    serde_json::to_string_pretty(
                        &serde_json::from_str(&xml_string)
                            .map_err(|e| ServerError::Serialization(e.to_string()))?,
                    )
                    .map_err(|e| ServerError::Serialization(e.to_string()))?
                }
                HttpBody::Yaml(yaml) => serde_json::to_string_pretty(yaml)
                    .map_err(|e| ServerError::Serialization(e.to_string()))?,
                HttpBody::Empty => return Err(ServerError::EmptyBody),
            },
            APPLICATION_XML => match self {
                HttpBody::Json(json) => {
                    // Convertir JSON a XML antes de serializar
                    xml_to_string(json).map_err(|e| ServerError::Serialization(e.to_string()))?
                }
                HttpBody::Xml(xml) => {
                    xml_to_string(xml).map_err(|e| ServerError::Serialization(e.to_string()))?
                }
                HttpBody::Yaml(yaml) => {
                    // Convertir YAML a JSON primero, luego a XML
                    let json_string = serde_json::to_string(yaml)
                        .map_err(|e| ServerError::Serialization(e.to_string()))?;
                    let json_value: serde_json::Value = serde_json::from_str(&json_string)
                        .map_err(|e| ServerError::Serialization(e.to_string()))?;
                    xml_to_string(&json_value)
                        .map_err(|e| ServerError::Serialization(e.to_string()))?
                }
                HttpBody::Empty => return Err(ServerError::EmptyBody),
            },
            APPLICATION_YAML => match self {
                HttpBody::Json(json) => serde_yaml::to_string(json)
                    .map_err(|e| ServerError::Serialization(e.to_string()))?,
                HttpBody::Xml(xml) => {
                    // Convertir XML a JSON primero, luego a YAML
                    let json_string = xml_to_string(xml)
                        .map_err(|e| ServerError::Serialization(e.to_string()))?;
                    let json_value: serde_json::Value = serde_json::from_str(&json_string)
                        .map_err(|e| ServerError::Serialization(e.to_string()))?;
                    serde_yaml::to_string(&json_value)
                        .map_err(|e| ServerError::Serialization(e.to_string()))?
                }
                HttpBody::Yaml(yaml) => serde_yaml::to_string(yaml)
                    .map_err(|e| ServerError::Serialization(e.to_string()))?,
                HttpBody::Empty => return Err(ServerError::EmptyBody),
            },
            _ => {
                return Err(ServerError::InvalidFormat(
                    "Unsupported application format".to_string(),
                ))
            }
        };

        // Guardamos el contenido serializado en el archivo
        if file.write_all(serialized.as_bytes()).is_err() {
            return Err(ServerError::SavePr);
        }
        Ok(())
    }

    pub fn from_string(application: &str, message: &str, key: &str) -> Result<Self, ServerError> {
        match application {
            APPLICATION_JSON => {
                let body_str = format!("{{\"{}\": \"{}\"}}", key, message);
                let json =
                    serde_json::from_str(&body_str).map_err(|_| ServerError::HttpParseJsonBody)?;
                Ok(HttpBody::Json(json))
            }
            APPLICATION_YAML | TEXT_YAML => {
                let body_str = format!("{}: \"{}\"", key, message);
                let yaml =
                    serde_yaml::from_str(&body_str).map_err(|_| ServerError::HttpParseYamlBody)?;
                Ok(HttpBody::Yaml(yaml))
            }
            APPLICATION_XML | TEXT_XML => {
                let body_str = format!("<{}>{}</{}>", key, message, key);
                let xml =
                    serde_xml_rs::from_str(&body_str).map_err(|_| ServerError::HttpParseXmlBody)?;
                Ok(HttpBody::Xml(xml))
            }
            _ => Err(ServerError::UnsupportedMediaType),
        }
    }

    /// Convierte el `HttpBody` al formato especificado por `content_type`.
    ///
    /// # Argumentos
    ///
    /// * `body` - Una referencia al `HttpBody` que se desea convertir.
    /// * `content_type` - Un string que indica el tipo de contenido deseado (e.g., "application/json", "application/xml", "application/yaml").
    ///
    /// # Retorno
    ///
    /// Retorna un `Result` que contiene un `HttpBody` convertido o un `ServerError` en caso de error.
    pub fn convert_body_to_content_type(
        body: HttpBody,
        content_type: &str,
    ) -> Result<HttpBody, ServerError> {
        match content_type {
            APPLICATION_JSON => match body {
                HttpBody::Json(_) => Ok(body), // Ya está en formato JSON
                HttpBody::Xml(xml) => Ok(HttpBody::Json(xml)),
                HttpBody::Yaml(yaml) => {
                    let json_str = serde_yaml::to_string(&yaml)
                        .map_err(|e| ServerError::Serialization(e.to_string()))?;
                    let json_value: JsonValue = serde_json::from_str(&json_str)
                        .map_err(|e| ServerError::Serialization(e.to_string()))?;
                    Ok(HttpBody::Json(json_value))
                }
                HttpBody::Empty => Ok(HttpBody::Empty),
            },
            APPLICATION_XML | TEXT_XML => match body {
                HttpBody::Json(json) => Ok(HttpBody::Xml(json)),
                HttpBody::Xml(_) => Ok(body), // Ya está en formato XML
                HttpBody::Yaml(yaml) => {
                    let json_str = serde_yaml::to_string(&yaml)
                        .map_err(|e| ServerError::Serialization(e.to_string()))?;
                    let json_value: JsonValue = serde_json::from_str(&json_str)
                        .map_err(|e| ServerError::Serialization(e.to_string()))?;
                    let xml_string = serde_xml_rs::to_string(&json_value)
                        .map_err(|e| ServerError::Serialization(e.to_string()))?;
                    let xml_value = serde_xml_rs::from_str(&xml_string)
                        .map_err(|e| ServerError::Serialization(e.to_string()))?;
                    Ok(HttpBody::Xml(xml_value))
                }
                HttpBody::Empty => Ok(HttpBody::Empty),
            },
            APPLICATION_YAML | TEXT_YAML => match body {
                HttpBody::Json(json) => {
                    let yaml_string = serde_yaml::to_string(&json)
                        .map_err(|e| ServerError::Serialization(e.to_string()))?;
                    let yaml_value: YamlValue = serde_yaml::from_str(&yaml_string)
                        .map_err(|e| ServerError::Serialization(e.to_string()))?;
                    Ok(HttpBody::Yaml(yaml_value))
                }
                HttpBody::Xml(xml) => {
                    let json_str = xml_to_string(&xml)
                        .map_err(|e| ServerError::Serialization(e.to_string()))?;
                    let json_value: JsonValue = serde_json::from_str(&json_str)
                        .map_err(|e| ServerError::Serialization(e.to_string()))?;
                    let yaml_string = serde_yaml::to_string(&json_value)
                        .map_err(|e| ServerError::Serialization(e.to_string()))?;
                    let yaml_value: YamlValue = serde_yaml::from_str(&yaml_string)
                        .map_err(|e| ServerError::Serialization(e.to_string()))?;
                    Ok(HttpBody::Yaml(yaml_value))
                }
                HttpBody::Yaml(_) => Ok(body), // Ya está en formato YAML
                HttpBody::Empty => Ok(HttpBody::Empty),
            },
            _ => Err(ServerError::InvalidFormat(
                "Unsupported content type".to_string(),
            )),
        }
    }

    pub fn get_value(&self, key: &str) -> JsonValue {
        match self {
            HttpBody::Json(json) => json[key].clone(),
            HttpBody::Xml(xml) => {
                let map: HashMap<String, JsonValue> = serde_json::from_value(xml.clone()).unwrap();
                map[key].clone()
            }
            HttpBody::Yaml(yaml) => {
                let map: HashMap<String, JsonValue> =
                    serde_json::from_value(serde_json::to_value(yaml).unwrap()).unwrap();
                map[key].clone()
            }
            HttpBody::Empty => JsonValue::Null,
        }
    }

    // fn convert_body_pr_in_string(&self) -> Result<String, ServerError> {
    //     let mut result = String::new();
    //     // let id:u64 = match self.get_value("id"){
    //     //     JsonValue::Number(id) => id.as_u64().unwrap(),
    //     //     _ => return Err(ServerError::HttpFieldNotFound("id".to_string())),
    //     // };
    //     // match self {
    //     //     HttpBody::Json(_) => result.push_str(&format!("id: {}", id)),
    //     //     HttpBody::Xml(_) => result.push_str(&format!("<id>{}</id>", id)),
    //     //     HttpBody::Yaml(_) => result.push_str(&format!("id: {}", id)),
    //     //     HttpBody::Empty => return Err(ServerError::HttpFieldNotFound("id".to_string())),
    //     // };
    //     println!("self: {:?}", self);
    //     let id = self.get_value("id");
    //     let owner = self.get_value("owner");
    //     let title = self.get_value("title");
    //     let body = self.get_value("body");
    //     let state = self.get_value("state");
    //     let base = self.get_value("base");
    //     let head = self.get_value("head");
    //     let repo = self.get_value("repo");
    //     let mergeable = self.get_value("mergeable");
    //     let changed_files = self.get_value("changed_files");
    //     let amount_commits = self.get_value("amount_commits");
    //     let commits = self.get_value("commits");

    //     match self {
    //         HttpBody::Json(_) => {
    //             result.push_str(&format!("{{\tid: {},\n\towner: {},\n\ttitle: {},\n\tbody: {},\n\tstate: {},\n\tbase: {},\n\thead: {},\n\trepo: {},\n\tmergeable: {},\n\tchanged_files: {},\n\tamount_commits: {},\n\tcommits: {}}}", id, owner, title, body, state, base, head, repo, mergeable, changed_files, amount_commits, commits));
    //         }
    //         HttpBody::Xml(_) => {
    //             result.push_str(&format!("<id>{}</id>\n<owner>{}</owner>\n<title>{}</title>\n<body>{}</body>\n<state>{}</state>\n<base>{}</base>\n<head>{}</head>\n<repo>{}</repo>\n<mergeable>{}</mergeable>\n<changed_files>{}</changed_files>\n<amount_commits>{}</amount_commits>\n<commits>{}</commits>", id, owner, title, body, state, base, head, repo, mergeable, changed_files, amount_commits, commits));
    //         }
    //         HttpBody::Yaml(_) => {
    //             result.push_str(&format!("id: {}\nowner: {}\ntitle: {}\nbody: {}\nstate: {}\nbase: {}\nhead: {}\nrepo: {}\nmergeable: {}\nchanged_files: {}\namount_commits: {}\ncommits: {}", id, owner, title, body, state, base, head, repo, mergeable, changed_files, amount_commits, commits));
    //         }
    //         HttpBody::Empty => return Err(ServerError::HttpFieldNotFound("owner".to_string())),
    //     };

    //     Ok(result)
    // }
}
