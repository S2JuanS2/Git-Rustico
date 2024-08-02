use std::{collections::HashMap, hash::{DefaultHasher, Hash, Hasher}};

use crate::servers::errors::ServerError;

use super::http_body::HttpBody;


/// Guarda el mapa de pull requests en un archivo JSON.
///
/// Esta función toma un mapa de pull requests, lo serializa en formato JSON, y lo escribe en el archivo
/// especificado por `pr_map_path`. Esto se utiliza para almacenar de manera persistente el estado actual
/// del mapa de pull requests, que mapea claves hash únicas a números de pull requests.
///
/// # Argumentos
///
/// * `pr_map_path` - La ruta del archivo donde se debe guardar el mapa de pull requests.
/// * `pr_map` - Un mapa de hash que contiene las claves hash de los pull requests como claves y el
///   número del pull request como valores.
///
/// # Retornos
///
/// Devuelve `Ok(())` si el mapa se guarda correctamente.
/// Devuelve `Err(ServerError::SaveMapPrFile)` si ocurre un error durante la serialización o escritura del archivo.
/// 
pub fn save_pr_map(pr_map_path: &str, pr_map: &HashMap<String, u64>) -> Result<(), ServerError> {
    let file_content = serde_json::to_string_pretty(pr_map).map_err(|_| ServerError::SaveMapPrFile)?;
    std::fs::write(pr_map_path, file_content).map_err(|_| ServerError::SaveMapPrFile)?;
    Ok(())
}

/// Genera una clave hash única a partir de las ramas head y base de un pull request.
///
/// Esta función crea una cadena única que representa un pull request específico combinando las ramas
/// `head` y `base`, y luego calculando un hash a partir de esa combinación. Este hash se utiliza como
/// clave en el mapa de pull requests para identificar de manera única cada pull request.
///
/// # Argumentos
///
/// * `head` - La rama de origen del pull request.
/// * `base` - La rama de destino del pull request.
///
/// # Retornos
///
/// Devuelve una cadena que representa la clave hash única para el pull request.
/// 
pub fn generate_head_base_hash(head: &str, base: &str) -> String {
    let mut hasher = DefaultHasher::new();
    format!("{}:{}", head, base).hash(&mut hasher);
    let hash = hasher.finish();
    format!("{:x}", hash)
}

/// Lee el mapa de pull requests desde un archivo JSON.
///
/// Esta función carga el contenido de un archivo JSON que contiene el mapa de pull requests, lo deserializa
/// y lo devuelve como un `HashMap`. Si el archivo no existe o está vacío, se devuelve un mapa vacío.
///
/// # Argumentos
///
/// * `pr_map_path` - La ruta del archivo desde donde se debe leer el mapa de pull requests.
///
/// # Retornos
///
/// Devuelve `Ok(HashMap<String, u64>)` con el mapa de pull requests si la lectura y deserialización son exitosas.
/// Devuelve `Err(ServerError::ReadMapPrFile)` si ocurre un error durante la lectura o deserialización del archivo.
/// 
pub fn read_pr_map(pr_map_path: &str) -> Result<HashMap<String, u64>, ServerError> {
    let file_content = std::fs::read_to_string(pr_map_path).unwrap_or_else(|_| "{}".to_string());
    serde_json::from_str(&file_content).map_err(|_| ServerError::ReadMapPrFile)
}


/// Actualiza y guarda el mapa de pull requests con un nuevo número de PR asignado.
///
/// Esta función inserta un nuevo par clave-valor en el mapa de pull requests, donde la clave es
/// una cadena hash única generada a partir de los datos del pull request, y el valor es el número
/// del pull request. Luego guarda el mapa actualizado en un archivo especificado.
///
/// # Argumentos
///
/// * `pr_map` - Un mapa mutable que contiene las claves hash de los pull requests existentes como claves
///   y el número del pull request como valores.
/// * `pr_map_path` - La ruta del archivo donde se debe guardar el mapa de pull requests.
/// * `hash_key` - La clave hash única que identifica el pull request.
/// * `pr_number` - El número del pull request que se asocia con la clave hash.
///
/// # Retornos
///
/// Devuelve `Ok(())` si el mapa se actualiza y guarda correctamente.
/// Devuelve `Err(ServerError)` si ocurre un error durante la actualización o el guardado del mapa.
/// 
pub fn update_pr_map(pr_map: &mut HashMap<String, u64>, pr_map_path: &str, hash_key: String, pr_number: u64) -> Result<(), ServerError> {
    pr_map.insert(hash_key, pr_number);
    save_pr_map(pr_map_path, pr_map)?;
    Ok(())
}

/// Genera una clave hash única para un pull request a partir de los campos 'head' y 'base'.
///
/// Esta función extrae los campos 'head' y 'base' del cuerpo de la solicitud HTTP y
/// genera un hash que se utiliza para identificar de forma única el pull request.
///
/// # Argumentos
///
/// * `body` - El cuerpo de la solicitud HTTP que contiene los datos del pull request.
///
/// # Retornos
///
/// Devuelve `Ok(String)` que representa la clave hash única.
/// Devuelve `Err(ServerError)` si hay un error al extraer los campos necesarios.
/// 
pub fn generate_pr_hash_key(body: &HttpBody) -> Result<String, ServerError> {
    let head = body.get_field("head")?;
    let base = body.get_field("base")?;
    Ok(generate_head_base_hash(&head, &base))
}

/// Verifica si un pull request ya existe en el mapa de pull requests utilizando la clave hash.
///
/// Esta función comprueba si la clave hash proporcionada ya está presente en el mapa de pull requests,
/// lo que indica que el pull request ya ha sido creado previamente.
///
/// # Argumentos
///
/// * `pr_map` - Un mapa que contiene las claves hash de los pull requests existentes como claves
///   y el número del pull request como valores.
/// * `hash_key` - La clave hash que se desea verificar.
///
/// # Retornos
///
/// Devuelve `true` si el pull request ya existe, `false` en caso contrario.
/// 
pub fn pr_already_exists(pr_map: &HashMap<String, u64>, hash_key: &String) -> bool {
    pr_map.contains_key(hash_key)
}