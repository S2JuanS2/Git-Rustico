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

// use super::errors::UtilError;

#[derive(Debug)]
pub struct GitServer {
    pub version: u32,
    pub capabilities: Vec<String>,
    pub shallow: Vec<String>,
    pub references: Vec<Reference>,
}

impl GitServer {
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
    pub fn new(content: &Vec<Vec<u8>>) -> Result<GitServer, UtilError> {
        let classified = AdvertisedRefLine::classify_vec(content)?;
        GitServer::from_classified(classified)
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
    fn from_classified(classified: Vec<AdvertisedRefLine>) -> Result<GitServer, UtilError> {
        let mut version: u32 = VERSION_DEFAULT;
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

        Ok(GitServer {
            version,
            capabilities,
            shallow,
            references,
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

    pub fn create_from_path(
        path_repo: &str,
        version: u32,
        capabilities: Vec<String>,
    ) -> Result<GitServer, UtilError> {
        let references = Reference::extract_references_from_git(path_repo)?;
        Ok(GitServer {
            version,
            capabilities,
            shallow: Vec::new(),
            references,
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
        for reference in &self.references {
            let reference = format!("{} {}\n", reference.get_hash(), reference.get_name());
            let reference = pkt_line::add_length_prefix(&reference, reference.len());
            // println!("Sending reference: {}", reference);
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

    pub fn update_data(&mut self, capabilities: Vec<String>, references: Vec<String>) {
        retain_common_values(&mut self.capabilities, &capabilities);
        filter_by_hash(&mut self.references, &references);
    }
}

fn filter_by_hash(references: &mut Vec<Reference>, refnames: &[String]) {
    references.retain(|reference| refnames.contains(reference.get_hash()));
}

fn retain_common_values(vec1: &mut Vec<String>, vec2: &[String]) {
    let set2: std::collections::HashSet<_> = vec2.iter().collect();

    vec1.retain(|item| set2.contains(item));
}
