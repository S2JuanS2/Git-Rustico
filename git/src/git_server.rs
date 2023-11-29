pub mod handle_references;
pub mod reference_information;

use std::io::Write;


use crate::{
    consts::VERSION_DEFAULT,
    git_transport::{advertised::AdvertisedRefLine, references::Reference},
    util::{
        connections::{send_flush, send_message},
        errors::UtilError,
        pkt_line,
    },
};

use crate::git_server::handle_references::HandleReferences;

#[derive(Debug)]
pub struct GitServer {
    pub src_repo: String,
    pub version: u32,
    pub capabilities: Vec<String>,
    pub shallow: Vec<String>,
    pub available_references: Vec<Reference>,
    handle_references: HandleReferences, // No tendra el Head
}

impl GitServer {
    /// Esta funcion es llamada del lado del CLIENTE.
    /// Crea una nueva estructura `GitServer` a partir del contenido proporcionado.
    ///
    /// # Descripción
    /// Esta función toma un vector de vectores de bytes (`content`) y lo clasifica en líneas
    /// de referencia anunciadas (`AdvertisedRefLine`). Luego, crea una nueva estructura `GitServer`
    /// llamando al método `from_classified`.
    ///
    /// # Argumentos
    /// * `content` - Un vector de vectores de bytes que representan las referencias anunciadas.
    ///
    /// # Retorno
    /// Devuelve un `Result` que contiene la estructura `GitServer` si la operación es exitosa,
    /// o un error de `UtilError` si ocurre algún problema durante el proceso.
    ///
    pub fn new(content: &Vec<Vec<u8>>, src_repo: &str, my_capabilities: &Vec<String>) -> Result<GitServer, UtilError> {
        let classified = AdvertisedRefLine::classify_vec(content)?;
        GitServer::from_classified(classified, src_repo, my_capabilities)
    }

    /// Construye una estructura `GitServer` a partir de líneas de referencia clasificadas.
    ///
    /// # Descripción
    /// Esta función toma un vector de líneas de referencia clasificadas (`classified`) y extrae
    /// información para construir una instancia de `GitServer`. Se asignan los valores de la
    /// versión, capacidades, referencias superficiales y referencias del conjunto de líneas clasificadas.
    ///
    /// # Argumentos
    /// * `classified` - Vector de líneas de referencia clasificadas a partir del contenido recibido.
    ///
    /// # Retorno
    /// Devuelve un `Result` que contiene la estructura `GitServer` si la operación es exitosa,
    /// o un error de `UtilError` si ocurre algún problema durante el proceso.
    ///
    fn from_classified(classified: Vec<AdvertisedRefLine>, src_repo: &str, my_capabilities: &Vec<String>) -> Result<GitServer, UtilError> {
        let mut version: u32 = VERSION_DEFAULT;
        let mut capabilities: Vec<String> = Vec::new();
        let mut shallow: Vec<String> = Vec::new();
        let mut available_references: Vec<Reference> = Vec::new();

        for line in classified {
            match line {
                AdvertisedRefLine::Version(v) => version = v,
                AdvertisedRefLine::Capabilities(c) => capabilities = c,
                AdvertisedRefLine::Shallow { obj_id } => shallow.push(obj_id),
                AdvertisedRefLine::Ref { obj_id, ref_name } => {
                    available_references.push(Reference::new(&obj_id, &ref_name)?)
                }
            }
        }
        
        GitServer::filter_capabilities(&mut capabilities, my_capabilities)?;
        Ok(GitServer {
            src_repo: src_repo.to_string(),
            version,
            capabilities,
            shallow,
            handle_references: HandleReferences::new_from_references(&available_references),
            available_references,
        })
    }

    /// Obtiene una referencia a la lista de referencias disponibles en `GitServer`.
    ///
    /// # Descripción
    /// Devuelve una referencia al vector que contiene las referencias disponibles.
    ///
    /// # Retorno
    /// Devuelve una referencia al vector que contiene las referencias disponibles.
    ///
    pub fn get_references(&self) -> &Vec<Reference> {
        &self.available_references
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
        self.available_references.get(index)
    }

    /// Esta funcion es llamada del lado del SERVIDOR.
    /// Crea una instancia de `GitServer` a partir de la ruta del repositorio y otros parámetros.
    ///
    /// Esta función crea una instancia de la estructura `GitServer` a partir de la ruta del
    /// repositorio, la versión del servidor Git, y las capacidades del servidor. Además, extrae
    /// las referencias del repositorio utilizando la función `Reference::extract_references_from_git`.
    ///
    /// # Argumentos
    ///
    /// * `path_repo` - Ruta del repositorio Git en el sistema de archivos.
    /// * `version`   - Número de versión del servidor Git.
    /// * `capabilities` - Vector que contiene las capacidades del servidor Git.
    ///
    /// # Retorno
    ///
    /// Retorna un `Result` con la instancia de `GitServer` en caso de éxito, o un `UtilError` en
    /// caso de error durante la extracción de las referencias del repositorio.
    ///
    pub fn create_from_path(
        path_repo: &str,
        version: u32,
        capabilities: &[String],
    ) -> Result<GitServer, UtilError> {
        let available_references = Reference::extract_references_from_git(path_repo)?;
        // GitServer::filter_capabilities(&mut capabilities, );
        
        Ok(GitServer {
            src_repo: path_repo.to_string(),
            version,
            capabilities: capabilities.to_vec(),
            shallow: Vec::new(),
            handle_references: HandleReferences::new_from_references(&available_references),
            available_references,
        })
    }

    pub fn send_references(&self, writer: &mut dyn Write) -> Result<(), UtilError> {
        // Send version
        let version = format!("version {}\n", self.version);
        let version = pkt_line::add_length_prefix(&version, version.len());
        send_message(
            writer,
            &version,
            UtilError::VersionNotSentDiscoveryReferences,
        )?;

        // Send references
        // HEAD lo inserte 1ero en el vector
        // Primera refer
        let mut firts_references = format!("{} {}\n", self.available_references[0].get_hash(), self.available_references[0].get_ref_path());
        if !self.capabilities.is_empty()
        {
            firts_references.push_str(&format!(" {}\n", self.capabilities.join(" ")));
            let firts_references = pkt_line::add_length_prefix(&firts_references, firts_references.len());
            send_message(writer, &firts_references, UtilError::ReferencesObtaining)?;
        }

        for reference in &self.available_references[1..] {
            let reference = format!("{} {}\n", reference.get_hash(), reference.get_ref_path());
            let reference = pkt_line::add_length_prefix(&reference, reference.len());
            send_message(writer, &reference, UtilError::ReferencesObtaining)?;
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

    /// Actualiza los datos del `GitServer` con nuevas capacidades y referencias.
    ///
    /// Esta función toma un vector de nuevas capacidades y referencias, y actualiza el `GitServer`
    /// reteniendo solo los valores comunes de las capacidades y filtrando las referencias que ya
    /// están presentes en el servidor.
    ///
    /// # Argumentos
    ///
    /// * `capabilities` - Vector que contiene las nuevas capacidades a ser consideradas.
    /// * `references`   - Vector que contiene las nuevas referencias a ser consideradas.
    ///
    pub fn update_data(&mut self, capabilities: Vec<String>, references: Vec<String>) {
        retain_common_values(&mut self.capabilities, &capabilities);
        filter_by_hash(&mut self.available_references, &references);
    }

    /// Actualiza los commits locales en las referencias del servidor.
    ///
    /// # Argumentos
    ///
    /// * `references`: Vector de referencias que se utilizará para actualizar los commits locales.
    ///
    pub fn update_local_references(&mut self, references: &Vec<Reference>) {
        self.handle_references.update_local_commit(references);
    }

    /// Obtiene las referencias remotas del servidor Git.
    ///
    /// # Errores
    ///
    /// Retorna un `Result` que puede contener un vector de referencias (`Ok(Vec<Reference>)`)
    /// o un error de utilidad (`Err(UtilError)`).
    ///
    pub fn get_remote_references(&self) -> Result<Vec<Reference>, UtilError> {
        self.handle_references.get_remote_references()
    }

    /// Obtiene las referencias locales almacenadas en el servidor Git.
    ///
    /// Itera sobre las referencias almacenadas internamente y crea un vector de
    /// referencias con la información local. Solo incluye las referencias que tienen
    /// un commit local definido. Retorna un `Result` que contiene el vector
    /// de referencias locales o un error de utilidad en caso de problemas.
    ///
    /// # Errores
    ///
    /// Retorna un `Result` que puede contener un vector de referencias (`Ok(Vec<Reference>)`)
    /// o un error de utilidad (`Err(UtilError)`).
    ///
    pub fn get_local_references(&self) -> Result<Vec<Reference>, UtilError> {
        self.handle_references.get_local_references()
    }

    /// Obtiene las capacidades del servidor Git.
    ///
    /// Retorna una referencia al vector de capacidades del servidor. Esta función
    /// proporciona acceso inmutable a las capacidades del servidor Git.
    ///
    pub fn get_capabilities(&self) -> &Vec<String> {
        &self.capabilities
    }

    /// Confirma las referencias locales del servidor Git en base a una lista de commits locales.
    ///
    /// Itera sobre las referencias almacenadas internamente y, si el commit local de
    /// la referencia está presente en la lista de commits locales proporcionada,
    /// confirma la referencia localmente.
    ///
    /// # Argumentos
    ///
    /// * `local_commits`: Vector de commits locales que se utilizará para confirmar referencias.
    ///
    pub fn confirm_local_references(&mut self, local_commits: &Vec<String>) {
        self.handle_references.confirm_local_references(local_commits);
    }

    /// Obtiene las referencias del servidor Git que necesitan ser actualizadas localmente.
    ///
    /// Itera sobre las referencias almacenadas internamente y crea un vector de
    /// referencias con la información remota de las referencias que han sido
    /// actualizadas localmente. Una referencia se considera actualizada si su
    /// commit local difiere del commit remoto. Retorna un `Result` que contiene el
    /// vector de referencias actualizadas o un error de utilidad en caso de problemas.
    ///
    /// # Errores
    ///
    /// Retorna un `Result` que puede contener un vector de referencias (`Ok(Vec<Reference>)`)
    /// o un error de utilidad (`Err(UtilError)`).
    ///
    pub fn get_references_for_updating(&self) -> Result<Vec<Reference>, UtilError> {
        self.handle_references.get_references_for_updating()
    }

    /// Filtra las capacidades del servidor manteniendo solo aquellas que coinciden con las capacidades del cliente.
    ///
    /// # Argumentos
    ///
    /// * `capabilities`: Vector mutable que contiene las capacidades del servidor.
    /// * `my_capabilities`: Vector de capacidades del cliente.
    ///
    /// # Errores
    ///
    /// Retorna un `Result` que contiene un mensaje de éxito (`Ok(())`) si las capacidades fueron filtradas
    /// exitosamente, o un error de utilidad (`Err(UtilError::ServerCapabilitiesNotSupported)`) si
    /// las capacidades del servidor no son compatibles con las del cliente.
    ///
    fn filter_capabilities(capabilities: &mut Vec<String>, my_capabilities: &Vec<String>) -> Result<(), UtilError>{
        retain_common_values(capabilities, my_capabilities);
        if capabilities.len() == my_capabilities.len() {
            Ok(())
        } else {
            Err(UtilError::ServerCapabilitiesNotSupported)
        }
    }
    /// Verifica si el servidor Git soporta la capacidad de "multi_ack".
    /// 
    pub fn is_multiack(&self) -> bool {
        self.capabilities.contains(&"multi_ack".to_string())
    }
    
}

/// Filtra las referencias basándose en un conjunto de hash de referencias.
///
/// Esta función toma un vector mutable de referencias y filtra las referencias que tienen un hash
/// presente en el vector `refnames`. Las referencias no presentes en `refnames` se eliminan del
/// vector de referencias.
///
/// # Argumentos
///
/// * `references` - Vector mutable de referencias a ser filtrado.
/// * `refs_hash` - Vector de hash de referencias usado para filtrar.
///
/// # Nota
///
/// Esta función es útil para mantener solo las referencias locales que también existen en el
/// servidor durante la actualización de datos del `GitServer`.
///
fn filter_by_hash(references: &mut Vec<Reference>, refs_hash: &[String]) {
    references.retain(|reference| refs_hash.contains(reference.get_hash()));
}

/// Retiene los valores comunes entre dos vectores de cadenas.
///
/// Esta función toma dos vectores de cadenas y retiene solo los elementos que son comunes entre
/// ambos vectores. Los elementos que no son comunes son eliminados del primer vector (`vec1`).
///
/// # Argumentos
///
/// * `vec1` - Vector mutable de cadenas que se retendrá solo con los elementos comunes.
/// * `vec2` - Vector de cadenas que se utiliza para determinar los elementos comunes.
///
/// # Nota
///
/// Esta función es útil para actualizar conjuntos de capacidades o referencias en el
/// `GitServer`, reteniendo solo los valores comunes entre el conjunto actual y el conjunto nuevo.
///
fn retain_common_values(vec1: &mut Vec<String>, vec2: &[String]) {
    let set2: std::collections::HashSet<_> = vec2.iter().collect();

    vec1.retain(|item| set2.contains(item));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_by_hash_should_retain_common_references() {
        // Crear algunas referencias para el ejemplo.
        let reference1 = Reference::new("hash1", "HEAD").unwrap();
        let reference2 = Reference::new("hash2", "refs/tags/v1").unwrap();
        let reference3 =
            Reference::new("hash3", "refs/heads/main").unwrap();

        // Crear un vector de referencias inicial.
        let mut references = vec![reference1.clone(), reference2.clone(), reference3.clone()];

        // Filtrar las referencias por hash, reteniendo solo las referencias comunes.
        filter_by_hash(&mut references, &["hash2".to_string(), "hash3".to_string()]);

        // Verificar que solo las referencias "hash2" y "hash3" permanezcan en el vector.
        assert_eq!(references, vec![reference2, reference3]);
    }

    #[test]
    fn filter_by_hash_should_retain_nothing_if_no_common_references() {
        // Crear algunas referencias para el ejemplo.
        let reference1 = Reference::new("hash1", "HEAD").unwrap();
        let reference2 = Reference::new("hash2", "HEAD").unwrap();
        let reference3 = Reference::new("hash3", "HEAD").unwrap();

        // Crear un vector de referencias inicial.
        let mut references = vec![reference1.clone(), reference2.clone(), reference3.clone()];

        // Filtrar las referencias por hash, no debería retener ninguna referencia.
        filter_by_hash(&mut references, &["hash4".to_string(), "hash5".to_string()]);

        // Verificar que el vector esté vacío después de la filtración.
        assert!(references.is_empty());
    }

    #[test]
    fn retain_common_values_should_retain_common_items() {
        // Crear dos vectores con algunos elementos en común.
        let mut vec1 = vec![
            "item1".to_string(),
            "item2".to_string(),
            "item3".to_string(),
        ];
        let vec2 = vec![
            "item2".to_string(),
            "item3".to_string(),
            "item4".to_string(),
        ];

        // Retener solo los elementos comunes entre los dos vectores.
        retain_common_values(&mut vec1, &vec2);

        // Verificar que solo los elementos "item2" y "item3" permanezcan en vec1.
        assert_eq!(vec1, vec!["item2".to_string(), "item3".to_string()]);
    }

    #[test]
    fn retain_common_values_should_retain_nothing_if_no_common_items() {
        // Crear dos vectores sin elementos en común.
        let mut vec1 = vec!["item1".to_string(), "item2".to_string()];
        let vec2 = vec!["item3".to_string(), "item4".to_string()];

        // Retener solo los elementos comunes entre los dos vectores, no debería retener nada.
        retain_common_values(&mut vec1, &vec2);

        // Verificar que vec1 esté vacío después de la retención.
        assert!(vec1.is_empty());
    }
}
