use crate::{consts::VERSION_DEFAULT, util::{errors::UtilError, validation::is_valid_obj_id, connections::{send_flush, send_message}, pkt_line}};

use std::{fmt, vec, io::Write};

use super::references::Reference;



/// Representa las referencias anunciadas recibidas durante una operación de fetch/push en Git.
///
/// Esta estructura encapsula la información sobre las referencias anunciadas por un servidor Git
/// en respuesta a operaciones de fetch o push. Proporciona detalles sobre las capacidades soportadas,
/// las referencias superficiales y las referencias disponibles en el repositorio en el servidor.
///
/// Esto es particularmente útil en el contexto de una implementación de servidor Git que utiliza
/// el protocolo de transporte de Git, donde el servidor responde con referencias anunciadas durante
/// las solicitudes de los clientes, como las operaciones de fetch o push.
/// Explicacion de los miembros
/// - `version`: La versión del protocolo Git que el servidor admite.
/// - `capabilities`: Una lista de capacidades soportadas por el servidor.
/// - `shallow`: Una lista de referencias superficiales.
/// - `references`: Una lista de referencias disponibles en el repositorio del servidor.
/// 
#[derive(Debug)]
pub struct AdvertisedRefs {
    pub version: u8,
    pub capabilities: Vec<String>,
    pub shallow: Vec<String>,
    pub references: Vec<Reference>,
}

impl AdvertisedRefs {
    /// Crea una nueva estructura `AdvertisedRefs` a partir del contenido proporcionado.
    ///
    /// # Descripción
    /// Esta función toma un vector de vectores de bytes (`content`) y lo clasifica en líneas
    /// de referencia anunciadas (`AdvertisedRefLine`). Luego, crea una nueva estructura `AdvertisedRefs`
    /// llamando al método `from_classified`.
    ///
    /// # Argumentos
    /// * `content` - Un vector de vectores de bytes que representan las referencias anunciadas.
    ///
    /// # Retorno
    /// Devuelve un `Result` que contiene la estructura `AdvertisedRefs` si la operación es exitosa,
    /// o un error de `UtilError` si ocurre algún problema durante el proceso.
    /// 
    pub fn new(content: &Vec<Vec<u8>>) -> Result<AdvertisedRefs, UtilError> {
        let classified = AdvertisedRefLine::classify_vec(content)?;
        AdvertisedRefs::from_classified(classified)
    }

    /// Construye una estructura `AdvertisedRefs` a partir de líneas de referencia clasificadas.
    ///
    /// # Descripción
    /// Esta función toma un vector de líneas de referencia clasificadas (`classified`) y extrae
    /// información para construir una instancia de `AdvertisedRefs`. Se asignan los valores de la
    /// versión, capacidades, referencias superficiales y referencias del conjunto de líneas clasificadas.
    ///
    /// # Argumentos
    /// * `classified` - Vector de líneas de referencia clasificadas a partir del contenido recibido.
    ///
    /// # Retorno
    /// Devuelve un `Result` que contiene la estructura `AdvertisedRefs` si la operación es exitosa,
    /// o un error de `UtilError` si ocurre algún problema durante el proceso.
    /// 
    fn from_classified(classified: Vec<AdvertisedRefLine>) -> Result<AdvertisedRefs, UtilError> {
        let mut version: u8 = VERSION_DEFAULT;
        let mut capabilities: Vec<String> = Vec::new();
        let mut shallow: Vec<String> = Vec::new();
        let mut references: Vec<Reference> = Vec::new();

        for line in classified {
            match line {
                AdvertisedRefLine::Version(v) => version = v,
                AdvertisedRefLine::Capabilities(c) => capabilities = c,
                AdvertisedRefLine::Shallow { obj_id } => shallow.push(obj_id),
                AdvertisedRefLine::Ref { obj_id, ref_name } => {
                    references.push(Reference::new(obj_id, ref_name)?)
                }
            }
        }

        Ok(AdvertisedRefs {
            version,
            capabilities,
            shallow,
            references,
        })
    }

    /// Obtiene una referencia a la lista de referencias disponibles en `AdvertisedRefs`.
    ///
    /// # Descripción
    /// Devuelve una referencia al vector que contiene las referencias disponibles.
    ///
    /// # Retorno
    /// Devuelve una referencia al vector que contiene las referencias disponibles.
    /// 
    pub fn get_references(&self) -> &Vec<Reference> {
        &self.references
    }

    /// Obtiene una referencia a una referencia específica en la lista por su índice.
    ///
    /// # Descripción
    /// Toma un índice como argumento y devuelve una referencia a la referencia en esa posición
    /// dentro del vector de referencias. Devuelve `None` si el índice está fuera de rango.
    ///
    /// # Argumentos
    /// * `index` - Índice de la referencia que se quiere obtener.
    ///
    /// # Retorno
    /// Devuelve una referencia a la referencia en la posición especificada si existe,
    /// de lo contrario, devuelve `None`.
    /// 
    pub fn get_reference(&self, index: usize) -> Option<&Reference> {
        self.references.get(index)
    }

    pub fn create_from_path(path_repo: &str, version: u8, capabilities: Vec<String>) -> Result<AdvertisedRefs, UtilError>
    {
        let references = Reference::extract_references_from_git(path_repo)?;
        Ok(AdvertisedRefs { version, capabilities, shallow: Vec::new(), references })
    }

    pub fn send_references(&self, writer: &mut dyn Write) -> Result<(), UtilError>
    {
        // Send version
        let version = format!("version {}\n", self.version);
        let version = pkt_line::add_length_prefix(&version, version.len());
        send_message(writer, version, UtilError::VersionNotSentDiscoveryReferences)?;

        // Send references
        // HEAD lo inserte 1ero en el vector
        for reference in &self.references {
            let reference = format!("{} {}\n", reference.get_hash(), reference.get_name());
            let reference = pkt_line::add_length_prefix(&reference, reference.len());
            println!("Sending reference: {}", reference);
            send_message(writer, reference, UtilError::ReferencesObtaining)?;
        }

        // Send shallow
        // for shallow in &self.shallow {
        //     let shallow = format!("shallow {}\n", shallow);
        //     let shallow = pkt_line::add_length_prefix(&shallow, shallow.len());
        //     send_message(writer, shallow, UtilError::ReferencesObtaining)?;
        // }

        send_flush(writer, UtilError::FlushNotSentDiscoveryReferences)?;
        Ok(())
    }
}

// pub struct Refere

/// `AdvertisedRefLine` es una enumeración que representa anuncios de referencias en el contexto de Git.
///
/// En Git, se utilizan anuncios de referencias para proporcionar información sobre las referencias disponibles,
/// capacidades del servidor y otros detalles relacionados con la comunicación entre clientes y servidores Git.
///
/// Esta enumeración puede ser clonada y mostrada con formato de depuración (`Debug`).
///
/// - `Version(u8)`: Anuncia la versión del servidor Git. Representa la versión del protocolo Git
///    que el servidor admite.
///
/// - `Capabilities(Vec<String>)`: Anuncia las capacidades admitidas por el servidor Git. Contiene una lista de
///    capacidades como cadenas de texto, que describen las características admitidas por el servidor.
///
/// - `Ref { obj_id: String, ref_name: String }`: Anuncia una referencia específica. Contiene el ID del objeto al
///    que apunta la referencia y el nombre de la referencia.
///
/// - `Shallow { obj_id: String }`: Anuncia una referencia "shallow" (superficial) en el servidor Git. Representa
///    una referencia "shallow" y contiene el ID del objeto superficial.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdvertisedRefLine {
    Version(u8),
    Capabilities(Vec<String>),
    Ref { obj_id: String, ref_name: String },
    Shallow { obj_id: String },
}

impl fmt::Display for AdvertisedRefLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AdvertisedRefLine::Version(version) => write!(f, "Version: {}", version),
            AdvertisedRefLine::Ref { obj_id, ref_name } => {
                write!(f, "Ref: (obj: {}, name: {})", obj_id, ref_name)
            }
            AdvertisedRefLine::Shallow { obj_id } => write!(f, "Shallow: {}", obj_id),
            AdvertisedRefLine::Capabilities(capabilities) => {
                write!(f, "Capabilities: {:?}", capabilities)
            }
        }
    }
}

impl AdvertisedRefLine {
    /// Clasifica y crea anuncios de referencias del servidor Git a partir de un vector de bytes.
    ///
    /// Esta función toma un vector de bytes que representa líneas de anuncios de referencias del servidor Git
    /// y genera anuncios correspondientes como un vector de `AdvertisedRefLine`. La función procesa cada línea del
    /// vector de bytes, intenta convertirla en una cadena de texto y clasifica las referencias utilizando la función
    /// `classify_server_refs`.
    ///
    /// # Argumentos
    ///
    /// # Retorno
    ///
    /// - `Ok(Vec<AdvertisedRefLine>)`: Si se procesan con éxito todas las líneas del vector de bytes y se generan
    ///   los anuncios apropiados, se devuelve un vector de anuncios de referencias.
    /// - `Err(UtilError)`: Si se produce un error al procesar las líneas o al clasificar las referencias, se devuelve
    ///   un error `UtilError` que indica el problema.
    ///
    pub fn classify_vec(content: &Vec<Vec<u8>>) -> Result<Vec<AdvertisedRefLine>, UtilError> {
        let mut result: Vec<AdvertisedRefLine> = Vec::new();
        for c in content {
            if let Ok(line_str) = std::str::from_utf8(c) {
                if let Ok(refs) = AdvertisedRefLine::classify_server_refs(line_str) {
                    result.extend(refs);
                }
            }
        }
        Ok(result)
    }

    /// Crea un anuncio de versión Git en función de la versión especificada.
    ///
    /// Esta función toma una cadena que representa la versión del servidor Git y genera un anuncio
    /// de versión correspondiente como un vector de `AdvertisedRefLine`. La versión se espera como una cadena,
    /// y se intenta analizar como un número entero sin signo de 8 bits (u8).
    ///
    /// ## Argumentos
    ///
    /// - `version`: Una cadena que representa la versión del servidor Git.
    ///
    /// ## Retorno
    ///
    /// - `Ok(vec![AdvertisedRefLine::Version(n)])`: Si la versión es 1 o 2, se genera un anuncio de versión con
    ///   el número correspondiente y se devuelve como un vector.
    /// - `Err(UtilError::InvalidVersionNumberError)`: Si la versión no es 1 ni 2, se genera un error `UtilError`
    ///   indicando que el número de versión es inválido.
    ///
    fn create_version(version: &str) -> Result<Vec<AdvertisedRefLine>, UtilError> {
        let version_number = version[..].parse::<u8>();
        match version_number {
            Ok(1) => Ok(vec![AdvertisedRefLine::Version(1)]),
            Ok(2) => Ok(vec![AdvertisedRefLine::Version(2)]),
            _ => Err(UtilError::InvalidVersionNumber),
        }
    }

    /// Crea un anuncio de referencia "shallow" en función del ID del objeto especificado.
    ///
    /// # Argumentos
    ///
    /// - `obj_id`: Una cadena que representa el ID del objeto.
    ///
    /// # Retorno
    ///
    /// - `Ok(vec![AdvertisedRefLine::Shallow { obj_id }])`: Si el ID del objeto es válido, se genera un anuncio de
    ///   referencia "shallow" con el ID del objeto proporcionado y se devuelve como un vector.
    /// - `Err(UtilError::InvalidObjectIdError)`: Si el ID del objeto no es válido, se genera un error `UtilError`
    ///   indicando que el ID del objeto es inválido.
    ///
    fn create_shallow(obj_id: &str) -> Result<Vec<AdvertisedRefLine>, UtilError> {
        if !is_valid_obj_id(obj_id) {
            return Err(UtilError::InvalidObjectId);
        }
        Ok(vec![AdvertisedRefLine::Shallow {
            obj_id: obj_id.to_string(),
        }])
    }

    /// Crea un anuncio de referencias Git basado en la entrada proporcionada.
    ///
    /// Esta función toma una cadena de entrada que representa un anuncio de referencias Git y genera
    /// un anuncio correspondiente como un vector de `AdvertisedRefLine`. La función puede manejar anuncios
    /// que contienen capacidades del servidor.
    ///
    /// # Argumentos
    ///
    /// - `input`: Una cadena que representa el anuncio de referencias Git.
    ///
    /// # Retorno
    ///
    /// - `Ok(vec![AdvertisedRefLine::Ref { obj_id, ref_name }])`: Si el anuncio de referencias Git no contiene
    ///   capacidades, se generan anuncios de referencias con los ID de objeto y nombres de referencia especificados
    ///   y se devuelven como un vector.
    /// - `Ok(vec![AdvertisedRefLine::Capabilities(caps), AdvertisedRefLine::Ref { obj_id, ref_name }])`: Si el anuncio
    ///   de referencias Git contiene capacidades, se generan anuncios de capacidades seguidos por anuncios de referencias
    ///   con los ID de objeto y nombres de referencia especificados, y se devuelven como un vector.
    /// - `Err(UtilError::InvalidObjectIdError)`: Si el anuncio de referencias Git es inválido o contiene una cantidad incorrecta
    ///   de partes, se genera un error `UtilError` indicando que el ID del objeto es inválido.
    ///
    fn create_ref(input: &str) -> Result<Vec<AdvertisedRefLine>, UtilError> {
        if !contains_capacity_list(input) {
            return _create_ref(input);
        }

        let parts: Vec<&str> = input.split('\0').collect();
        if parts.len() != 2 {
            return Err(UtilError::InvalidObjectId);
        }

        let mut vec: Vec<AdvertisedRefLine> = _create_ref(parts[0])?;
        vec.insert(0, extract_capabilities(parts[1])?);
        Ok(vec)
    }

    /// Clasifica y crea anuncios de referencias del servidor Git en función de la entrada proporcionada.
    ///
    /// Esta función toma una cadena de entrada que representa un anuncio de referencias del servidor Git y genera
    /// anuncios correspondientes como un vector de `AdvertisedRefLine`. La función clasifica la entrada en función
    /// de su contenido y llama a funciones específicas para crear los anuncios apropiados.
    ///
    /// # Argumentos
    ///
    /// - `input`: Una cadena que representa el anuncio de referencias del servidor Git.
    ///
    /// # Retorna
    ///
    /// - `Ok(vec![Anuncios de referencias])`: Si la entrada se clasifica correctamente y se generan los anuncios
    ///   apropiados, se devuelve un vector de anuncios de referencias.
    /// - `Err(UtilError::InvalidServerReferenceError)`: Si la entrada no se puede clasificar o es inválida, se genera un
    ///   error `UtilError` indicando que la referencia del servidor es inválida.
    ///
    fn classify_server_refs(input: &str) -> Result<Vec<AdvertisedRefLine>, UtilError> {
        let parts: Vec<&str> = input.split_whitespace().collect();

        if parts.len() == 1 {
            return Err(UtilError::InvalidServerReference);
        }

        // Verificar si el primer elemento es una versión válida
        if parts[0] == "version" {
            return AdvertisedRefLine::create_version(parts[1]);
        }
        // Verificar si el primer elemento es "shallow"
        if parts[0] == "shallow" {
            return AdvertisedRefLine::create_shallow(parts[1]);
        }

        // Verificar si el segundo elemento parece ser una referencia
        if parts[1].starts_with("refs/") || parts[1].starts_with("HEAD") {
            return AdvertisedRefLine::create_ref(input);
        }
        Err(UtilError::InvalidServerReference)
    }
}

fn extract_capabilities(input: &str) -> Result<AdvertisedRefLine, UtilError> {
    let mut capabilities: Vec<String> = Vec::new();
    capabilities.extend(input.split_whitespace().map(String::from));
    if capabilities.is_empty() {
        return Err(UtilError::InvalidServerReference);
    }
    Ok(AdvertisedRefLine::Capabilities(capabilities))
}

fn contains_capacity_list(input: &str) -> bool {
    input.contains('\0')
}

fn _create_ref(input: &str) -> Result<Vec<AdvertisedRefLine>, UtilError> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() != 2 {
        return Err(UtilError::InvalidServerReference);
    }
    if !is_valid_obj_id(parts[0]) {
        return Err(UtilError::InvalidObjectId);
    }
    Ok(vec![AdvertisedRefLine::Ref {
        obj_id: parts[0].to_string(),
        ref_name: parts[1].to_string(),
    }])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_version_valid_1() {
        let result = AdvertisedRefLine::create_version("1").unwrap();
        assert_eq!(result, vec![AdvertisedRefLine::Version(1)]);
    }

    #[test]
    fn test_create_version_valid_2() {
        let result = AdvertisedRefLine::create_version("2").unwrap();
        assert_eq!(result, vec![AdvertisedRefLine::Version(2)]);
    }

    #[test]
    fn test_create_version_invalid() {
        let invalid_result = AdvertisedRefLine::create_version("3");
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_create_version_non_numeric() {
        let invalid_result = AdvertisedRefLine::create_version("invalid");
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_create_shallow_valid() {
        let result =
            AdvertisedRefLine::create_shallow("1d3fcd5ced445d1abc402225c0b8a1299641f497").unwrap();
        assert_eq!(
            result,
            vec![AdvertisedRefLine::Shallow {
                obj_id: "1d3fcd5ced445d1abc402225c0b8a1299641f497".to_string()
            }]
        );
    }

    #[test]
    fn test_create_shallow_invalid() {
        let invalid_result = AdvertisedRefLine::create_shallow("invalid_id");
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_create_ref_without_capabilities() {
        let input = "1d3fcd5ced445d1abc402225c0b8a1299641f497 master";
        let result = AdvertisedRefLine::create_ref(input).unwrap();

        assert_eq!(
            result,
            vec![AdvertisedRefLine::Ref {
                obj_id: "1d3fcd5ced445d1abc402225c0b8a1299641f497".to_string(),
                ref_name: "master".to_string(),
            }]
        );
    }

    #[test]
    fn test_create_ref_without_capabilities_empty() {
        let input = "1d3fcd5ced445d1abc402225c0b8a1299641f497 master\0";
        let invalid_result = AdvertisedRefLine::create_ref(input);
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_create_ref_with_capabilities() {
        let input = "1d3fcd5ced445d1abc402225c0b8a1299641f497 master\0cap1 cap2";
        let result = AdvertisedRefLine::create_ref(input).unwrap();

        assert_eq!(
            result,
            vec![
                AdvertisedRefLine::Capabilities(vec!["cap1".to_string(), "cap2".to_string()]),
                AdvertisedRefLine::Ref {
                    obj_id: "1d3fcd5ced445d1abc402225c0b8a1299641f497".to_string(),
                    ref_name: "master".to_string(),
                }
            ]
        );
    }

    #[test]
    fn test_create_ref_invalid() {
        let input = "invalid_data";
        let invalid_result = AdvertisedRefLine::create_ref(input);
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_classify_server_refs_version() {
        let input = "version 2";
        let result = AdvertisedRefLine::classify_server_refs(input).unwrap();
        let expected = AdvertisedRefLine::create_version("2").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_classify_server_refs_shallow() {
        let input = "shallow 7217a7c7e582c46cec22a130adf4b9d7d950fba0";
        let result = AdvertisedRefLine::classify_server_refs(input).unwrap();
        let expected =
            AdvertisedRefLine::create_shallow("7217a7c7e582c46cec22a130adf4b9d7d950fba0").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_classify_server_refs_ref() {
        // Clasificar y crear un anuncio de referencia del servidor Git.
        let input = "7217a7c7e582c46cec22a130adf4b9d7d950fba0 refs/heads/master";
        let result = AdvertisedRefLine::classify_server_refs(input).unwrap();
        let expected = AdvertisedRefLine::create_ref(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_classify_server_refs_invalid() {
        let input = "invalid_data";
        let invalid_result = AdvertisedRefLine::classify_server_refs(input);
        assert!(invalid_result.is_err());
    }
}
