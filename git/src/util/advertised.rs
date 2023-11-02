use super::{errors::UtilError, validation::is_valid_obj_id};
use std::{fmt, vec};

/// `AdvertisedRefs` es una enumeración que representa anuncios de referencias en el contexto de Git.
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
pub enum AdvertisedRefs {
    Version(u8),
    Capabilities(Vec<String>),
    Ref { obj_id: String, ref_name: String },
    Shallow { obj_id: String },
}

impl fmt::Display for AdvertisedRefs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AdvertisedRefs::Version(version) => write!(f, "Version: {}", version),
            AdvertisedRefs::Ref { obj_id, ref_name } => {
                write!(f, "Ref: (obj: {}, name: {})", obj_id, ref_name)
            }
            AdvertisedRefs::Shallow { obj_id } => write!(f, "Shallow: {}", obj_id),
            AdvertisedRefs::Capabilities(capabilities) => {
                write!(f, "Capabilities: {:?}", capabilities)
            }
        }
    }
}

impl AdvertisedRefs {
    /// Clasifica y crea anuncios de referencias del servidor Git a partir de un vector de bytes.
    ///
    /// Esta función toma un vector de bytes que representa líneas de anuncios de referencias del servidor Git
    /// y genera anuncios correspondientes como un vector de `AdvertisedRefs`. La función procesa cada línea del
    /// vector de bytes, intenta convertirla en una cadena de texto y clasifica las referencias utilizando la función
    /// `classify_server_refs`.
    ///
    /// # Argumentos
    ///
    /// # Retorno
    ///
    /// - `Ok(Vec<AdvertisedRefs>)`: Si se procesan con éxito todas las líneas del vector de bytes y se generan
    ///   los anuncios apropiados, se devuelve un vector de anuncios de referencias.
    /// - `Err(UtilError)`: Si se produce un error al procesar las líneas o al clasificar las referencias, se devuelve
    ///   un error `UtilError` que indica el problema.
    ///
    pub fn classify_vec(content: &Vec<Vec<u8>>) -> Result<Vec<AdvertisedRefs>, UtilError> {
        let mut result: Vec<AdvertisedRefs> = Vec::new();
        for c in content {
            if let Ok(line_str) = std::str::from_utf8(c) {
                if let Ok(refs) = AdvertisedRefs::classify_server_refs(line_str) {
                    result.extend(refs);
                }
            }
        }
        Ok(result)
    }

    /// Crea un anuncio de versión Git en función de la versión especificada.
    ///
    /// Esta función toma una cadena que representa la versión del servidor Git y genera un anuncio
    /// de versión correspondiente como un vector de `AdvertisedRefs`. La versión se espera como una cadena,
    /// y se intenta analizar como un número entero sin signo de 8 bits (u8).
    ///
    /// ## Argumentos
    ///
    /// - `version`: Una cadena que representa la versión del servidor Git.
    ///
    /// ## Retorno
    ///
    /// - `Ok(vec![AdvertisedRefs::Version(n)])`: Si la versión es 1 o 2, se genera un anuncio de versión con
    ///   el número correspondiente y se devuelve como un vector.
    /// - `Err(UtilError::InvalidVersionNumberError)`: Si la versión no es 1 ni 2, se genera un error `UtilError`
    ///   indicando que el número de versión es inválido.
    ///
    fn create_version(version: &str) -> Result<Vec<AdvertisedRefs>, UtilError> {
        let version_number = version[..].parse::<u8>();
        match version_number {
            Ok(1) => Ok(vec![AdvertisedRefs::Version(1)]),
            Ok(2) => Ok(vec![AdvertisedRefs::Version(2)]),
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
    /// - `Ok(vec![AdvertisedRefs::Shallow { obj_id }])`: Si el ID del objeto es válido, se genera un anuncio de
    ///   referencia "shallow" con el ID del objeto proporcionado y se devuelve como un vector.
    /// - `Err(UtilError::InvalidObjectIdError)`: Si el ID del objeto no es válido, se genera un error `UtilError`
    ///   indicando que el ID del objeto es inválido.
    ///
    fn create_shallow(obj_id: &str) -> Result<Vec<AdvertisedRefs>, UtilError> {
        if !is_valid_obj_id(obj_id) {
            return Err(UtilError::InvalidObjectId);
        }
        Ok(vec![AdvertisedRefs::Shallow {
            obj_id: obj_id.to_string(),
        }])
    }

    /// Crea un anuncio de referencias Git basado en la entrada proporcionada.
    ///
    /// Esta función toma una cadena de entrada que representa un anuncio de referencias Git y genera
    /// un anuncio correspondiente como un vector de `AdvertisedRefs`. La función puede manejar anuncios
    /// que contienen capacidades del servidor.
    ///
    /// # Argumentos
    ///
    /// - `input`: Una cadena que representa el anuncio de referencias Git.
    ///
    /// # Retorno
    ///
    /// - `Ok(vec![AdvertisedRefs::Ref { obj_id, ref_name }])`: Si el anuncio de referencias Git no contiene
    ///   capacidades, se generan anuncios de referencias con los ID de objeto y nombres de referencia especificados
    ///   y se devuelven como un vector.
    /// - `Ok(vec![AdvertisedRefs::Capabilities(caps), AdvertisedRefs::Ref { obj_id, ref_name }])`: Si el anuncio
    ///   de referencias Git contiene capacidades, se generan anuncios de capacidades seguidos por anuncios de referencias
    ///   con los ID de objeto y nombres de referencia especificados, y se devuelven como un vector.
    /// - `Err(UtilError::InvalidObjectIdError)`: Si el anuncio de referencias Git es inválido o contiene una cantidad incorrecta
    ///   de partes, se genera un error `UtilError` indicando que el ID del objeto es inválido.
    ///
    fn create_ref(input: &str) -> Result<Vec<AdvertisedRefs>, UtilError> {
        if !contains_capacity_list(input) {
            return _create_ref(input);
        }

        let parts: Vec<&str> = input.split('\0').collect();
        if parts.len() != 2 {
            return Err(UtilError::InvalidObjectId);
        }

        let mut vec: Vec<AdvertisedRefs> = _create_ref(parts[0])?;
        vec.insert(0, extract_capabilities(parts[1])?);
        Ok(vec)
    }

    /// Clasifica y crea anuncios de referencias del servidor Git en función de la entrada proporcionada.
    ///
    /// Esta función toma una cadena de entrada que representa un anuncio de referencias del servidor Git y genera
    /// anuncios correspondientes como un vector de `AdvertisedRefs`. La función clasifica la entrada en función
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
    fn classify_server_refs(input: &str) -> Result<Vec<AdvertisedRefs>, UtilError> {
        let parts: Vec<&str> = input.split_whitespace().collect();

        if parts.len() == 1 {
            return Err(UtilError::InvalidServerReference);
        }

        // Verificar si el primer elemento es una versión válida
        if parts[0] == "version" {
            return AdvertisedRefs::create_version(parts[1]);
        }
        // Verificar si el primer elemento es "shallow"
        if parts[0] == "shallow" {
            return AdvertisedRefs::create_shallow(parts[1]);
        }

        // Verificar si el segundo elemento parece ser una referencia
        if parts[1].starts_with("refs/") || parts[1].starts_with("HEAD") {
            return AdvertisedRefs::create_ref(input);
        }
        Err(UtilError::InvalidServerReference)
    }
}

fn extract_capabilities(input: &str) -> Result<AdvertisedRefs, UtilError> {
    let mut capabilities: Vec<String> = Vec::new();
    capabilities.extend(input.split_whitespace().map(String::from));
    if capabilities.is_empty() {
        return Err(UtilError::InvalidServerReference);
    }
    Ok(AdvertisedRefs::Capabilities(capabilities))
}

fn contains_capacity_list(input: &str) -> bool {
    input.contains('\0')
}

fn _create_ref(input: &str) -> Result<Vec<AdvertisedRefs>, UtilError> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() != 2 {
        return Err(UtilError::InvalidServerReference);
    }
    if !is_valid_obj_id(parts[0]) {
        return Err(UtilError::InvalidObjectId);
    }
    Ok(vec![AdvertisedRefs::Ref {
        obj_id: parts[0].to_string(),
        ref_name: parts[1].to_string(),
    }])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_version_valid_1() {
        let result = AdvertisedRefs::create_version("1").unwrap();
        assert_eq!(result, vec![AdvertisedRefs::Version(1)]);
    }

    #[test]
    fn test_create_version_valid_2() {
        let result = AdvertisedRefs::create_version("2").unwrap();
        assert_eq!(result, vec![AdvertisedRefs::Version(2)]);
    }

    #[test]
    fn test_create_version_invalid() {
        let invalid_result = AdvertisedRefs::create_version("3");
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_create_version_non_numeric() {
        let invalid_result = AdvertisedRefs::create_version("invalid");
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_create_shallow_valid() {
        let result =
            AdvertisedRefs::create_shallow("1d3fcd5ced445d1abc402225c0b8a1299641f497").unwrap();
        assert_eq!(
            result,
            vec![AdvertisedRefs::Shallow {
                obj_id: "1d3fcd5ced445d1abc402225c0b8a1299641f497".to_string()
            }]
        );
    }

    #[test]
    fn test_create_shallow_invalid() {
        let invalid_result = AdvertisedRefs::create_shallow("invalid_id");
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_create_ref_without_capabilities() {
        let input = "1d3fcd5ced445d1abc402225c0b8a1299641f497 master";
        let result = AdvertisedRefs::create_ref(input).unwrap();

        assert_eq!(
            result,
            vec![AdvertisedRefs::Ref {
                obj_id: "1d3fcd5ced445d1abc402225c0b8a1299641f497".to_string(),
                ref_name: "master".to_string(),
            }]
        );
    }

    #[test]
    fn test_create_ref_without_capabilities_empty() {
        let input = "1d3fcd5ced445d1abc402225c0b8a1299641f497 master\0";
        let invalid_result = AdvertisedRefs::create_ref(input);
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_create_ref_with_capabilities() {
        let input = "1d3fcd5ced445d1abc402225c0b8a1299641f497 master\0cap1 cap2";
        let result = AdvertisedRefs::create_ref(input).unwrap();

        assert_eq!(
            result,
            vec![
                AdvertisedRefs::Capabilities(vec!["cap1".to_string(), "cap2".to_string()]),
                AdvertisedRefs::Ref {
                    obj_id: "1d3fcd5ced445d1abc402225c0b8a1299641f497".to_string(),
                    ref_name: "master".to_string(),
                }
            ]
        );
    }

    #[test]
    fn test_create_ref_invalid() {
        let input = "invalid_data";
        let invalid_result = AdvertisedRefs::create_ref(input);
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_classify_server_refs_version() {
        let input = "version 2";
        let result = AdvertisedRefs::classify_server_refs(input).unwrap();
        let expected = AdvertisedRefs::create_version("2").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_classify_server_refs_shallow() {
        let input = "shallow 7217a7c7e582c46cec22a130adf4b9d7d950fba0";
        let result = AdvertisedRefs::classify_server_refs(input).unwrap();
        let expected =
            AdvertisedRefs::create_shallow("7217a7c7e582c46cec22a130adf4b9d7d950fba0").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_classify_server_refs_ref() {
        // Clasificar y crear un anuncio de referencia del servidor Git.
        let input = "7217a7c7e582c46cec22a130adf4b9d7d950fba0 refs/heads/master";
        let result = AdvertisedRefs::classify_server_refs(input).unwrap();
        let expected = AdvertisedRefs::create_ref(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_classify_server_refs_invalid() {
        let input = "invalid_data";
        let invalid_result = AdvertisedRefs::classify_server_refs(input);
        assert!(invalid_result.is_err());
    }
}
