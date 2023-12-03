use std::collections::HashMap;

use crate::{git_transport::references::{Reference, ReferenceType}, util::errors::UtilError};

use super::reference_information::ReferenceInformation;

/// Estructura que maneja información sobre referencias en un repositorio Git.
/// Miembros:
/// - `references`: HashMap que almacena información sobre las referencias en el repositorio.
/// Explicacion de HashMap:
/// - `clave`: Path de la referencia.
/// - `valor`: Información sobre la referencia.
/// 
#[derive(Debug)]
pub struct HandleReferences {
    references: HashMap<String, ReferenceInformation>,
}

impl HandleReferences {
    /// Crea una nueva instancia de `HandleReferences` a partir de una lista de referencias.
    ///
    /// Esta función toma un vector de referencias y crea un mapa interno de `ReferenceInformation`.
    /// Las referencias de tipo `Head` se omiten durante la creación.
    ///
    /// # Argumentos
    ///
    /// * `references`: Vector de referencias que se utilizarán para inicializar `HandleReferences`.
    ///
    /// 
    pub fn new_from_references(references: &Vec<Reference>) -> Self {
        let mut references_map: HashMap<String, ReferenceInformation> = HashMap::new();

        for reference in references {
            // Ignore Head
            if reference.get_type() == ReferenceType::Head {
                continue;
            }

            references_map.insert(
                reference.get_ref_path().to_string(),
                ReferenceInformation::new(reference.get_hash(), None),
            );
        }

        HandleReferences {
            references: references_map,
        }
    }
    /// Actualiza los commits locales en las referencias según un vector de referencias dado.
    ///
    /// Itera sobre el vector de referencias dado y actualiza los commits locales en
    /// las referencias internas de `HandleReferences`. Si la referencia no existe en
    /// `HandleReferences`, se ignora.
    ///
    /// # Argumentos
    ///
    /// * `references`: Vector de referencias que se utilizará para actualizar los commits locales.
    ///
    pub fn update_local_commit(&mut self, references: &Vec<Reference>) {
        for reference in references {
            if let Some(reference_status) = self.references.get_mut(reference.get_ref_path()) {
                reference_status.update_local_commit(Some(reference.get_hash().trim().to_string()));
            }
        }
    }

    /// Obtiene las referencias remotas almacenadas en `HandleReferences`.
    ///
    /// Itera sobre las referencias almacenadas internamente y crea un vector de
    /// referencias con la información remota. Retorna un `Result` que contiene el vector
    /// de referencias o un error de utilidad en caso de problemas.
    ///
    /// # Errores
    ///
    /// Retorna un `Result` que puede contener un vector de referencias (`Ok(Vec<Reference>)`)
    /// o un error de utilidad (`Err(UtilError)`).
    ///
    pub fn get_remote_references(&self) -> Result<Vec<Reference>, UtilError> {
        let mut references: Vec<Reference> = Vec::new();

        for (path, value) in &self.references {
            let reference = Reference::new(value.get_remote_commit(), path)?;
            references.push(reference);
        }
        Ok(references)
    }

    /// Obtiene las referencias locales almacenadas en `HandleReferences`.
    ///
    /// Itera sobre las referencias almacenadas internamente y crea un vector de
    /// referencias con la información local. Solo incluye las referencias que tienen
    /// un commit local definido. Retorna un `Result` que contiene el vector
    /// de referencias o un error de utilidad en caso de problemas.
    ///
    /// # Errores
    ///
    /// Retorna un `Result` que puede contener un vector de referencias (`Ok(Vec<Reference>)`)
    /// o un error de utilidad (`Err(UtilError)`).
    ///
    pub fn get_local_references(&self) -> Result<Vec<Reference>, UtilError> {
        let mut references: Vec<Reference> = Vec::new();

        for (path, value) in &self.references {
            if let Some(local_commit) = &value.get_local_commit() {
                let reference = Reference::new(local_commit, path)?;
                references.push(reference);
            }
        }
        Ok(references)
    }

    /// Confirma las referencias locales en base a una lista de commits locales.
    ///
    /// Itera sobre las referencias almacenadas internamente y, si el commit local de
    /// la referencia está presente en la lista de commits locales proporcionada,
    /// confirma la referencia localmente.
    ///
    /// # Argumentos
    ///
    /// * `local_commits`: Vector de commits locales que se utilizará para confirmar referencias.
    ///
    pub fn confirm_local_references(&mut self, local_commits: &[String])
    {
        for value in self.references.values_mut() {
            if let Some(local_commit) = value.get_local_commit() {
                if local_commits.contains(&local_commit.to_string()) {
                    value.confirm_local_commit();
                }
            }
        }
    }

    /// Obtiene las referencias actualizadas en `HandleReferences`.
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
        let mut references: Vec<Reference> = Vec::new();

        for (path, value) in &self.references {
            if let Some(local_commit) = value.get_local_commit() {
                if local_commit == value.get_remote_commit() {
                    continue;
                }
            }
            let reference = Reference::new(value.get_remote_commit(), path)?;
            references.push(reference);
        }
        Ok(references)
    }

    /// Filtra las referencias del servidor para actualización basado en una lista de rutas de referencias.
    /// Asi solo actualizamos lo que queremos actualizar.
    ///
    /// # Argumentos
    ///
    /// * `path_references`: Vec<String> que contiene las rutas de las referencias que queremos actualizar.
    ///
    /// # Devuelve
    ///
    /// Un `Result<(), UtilError>` que indica si la operación fue exitosa o si ocurrió un error al filtrar
    /// las referencias. En caso de error, se proporciona un detalle específico en el tipo `UtilError`.
    ///
    pub fn update_references_filtering(&mut self, path_references: Vec<String>) -> Result<(), UtilError> {
        let mut new_refences: HashMap<String, ReferenceInformation> = HashMap::new();
        for path in path_references {
            if let Some(reference) = self.references.get(&path) {
                let local_commit = match reference.get_local_commit() {
                    Some(commit) => Some(commit.to_string()),
                    None => None,
                };
                new_refences.insert(path, ReferenceInformation::new(reference.get_remote_commit(), local_commit));
            }
        }
        self.references = new_refences;
        Ok(())
    }

    /// Verifica si una referencia específica está presente en las referencias del servidor.
    ///
    /// # Argumentos
    ///
    /// * `path`: La ruta de la referencia que se busca en las referencias del servidor.
    ///
    /// # Devuelve
    ///
    /// `true` si la referencia está presente en las referencias del servidor, `false` de lo contrario.
    ///
    pub fn contains_reference(&self, path: &str) -> bool {
        self.references.contains_key(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Función de utilidad para crear una referencia ficticia para pruebas.
    fn create_references() -> Vec<Reference> {
        vec![
            Reference::new("abc123", "refs/heads/main").unwrap(),
            Reference::new("def456", "refs/heads/feature").unwrap(),
            Reference::new("ghi789", "refs/tags/v1.0.0").unwrap(),
            Reference::new("ghi789", "HEAD").unwrap(),
        ]
    }

    #[test]
    fn test_new_from_references() {
        let references = create_references();
        let handle_references = HandleReferences::new_from_references(&references);

        // Verifica que las referencias se hayan almacenado correctamente en HandleReferences
        assert_eq!(handle_references.references.len(), 3); // Head se debería omitir
        assert!(handle_references.references.contains_key("refs/heads/main"));
        assert!(handle_references.references.contains_key("refs/heads/feature"));
        assert!(handle_references.references.contains_key("refs/tags/v1.0.0"));
    }

    #[test]
    fn test_update_local_commit() {
        let references = create_references();
        let mut handle_references = HandleReferences::new_from_references(&references);
        let references = vec![
            Reference::new("newcommit1", "refs/heads/main").unwrap(),
            Reference::new("newcommit2", "refs/heads/feature").unwrap(),
        ];

        handle_references.update_local_commit(&references);

        // Verifica que los commits locales se hayan actualizado correctamente
        assert_eq!(handle_references.references["refs/heads/main"].get_local_commit(), Some("newcommit1"));
        assert_eq!(handle_references.references["refs/heads/feature"].get_local_commit(), Some("newcommit2"));
    }

    #[test]
    fn test_get_remote_references() {
        let references = create_references();
        let handle_references = HandleReferences::new_from_references(&references);

        let remote_references = handle_references.get_remote_references().unwrap();
        let path_references: Vec<String> = remote_references.iter().map(|reference| reference.get_ref_path().to_string()).collect();
        // Verifica que las referencias remotas se hayan obtenido correctamente
        assert!(path_references.contains(&"refs/heads/main".to_string()));
        assert!(path_references.contains(&"refs/heads/feature".to_string()));
        assert!(path_references.contains(&"refs/tags/v1.0.0".to_string()));
    }

    #[test]
    fn test_get_local_references() {
        let references = create_references();
        let mut handle_references = HandleReferences::new_from_references(&references);
        let references = vec![
            Reference::new("newcommit1", "refs/heads/main").unwrap(),
            Reference::new("newcommit2", "refs/heads/feature").unwrap(),
        ];

        handle_references.update_local_commit(&references);
        let local_references = handle_references.get_local_references().unwrap();
        let path_references: Vec<String> = local_references.iter().map(|reference| reference.get_ref_path().to_string()).collect();

        // Verifica que las referencias locales se hayan obtenido correctamente
        assert_eq!(path_references.len(), 2); // Solo la referencia con commit local
        assert!(path_references.contains(&"refs/heads/main".to_string()));
        assert!(path_references.contains(&"refs/heads/feature".to_string()));
    }

    #[test]
    fn test_confirm_local_references() {
        let references = create_references();
        let mut handle_references = HandleReferences::new_from_references(&references);
        
        let references = vec![
            Reference::new("newcommit1", "refs/heads/main").unwrap(),
            Reference::new("newcommit2", "refs/heads/feature").unwrap(),
        ];

        handle_references.update_local_commit(&references);

        let local_commits_to_confirm = vec!["newcommit1".to_string(), "newcommit2".to_string()];
        handle_references.confirm_local_references(&local_commits_to_confirm);

        // Verifica que los commits locales se hayan confirmado correctamente
        assert!(handle_references.references["refs/heads/main"].is_confirmed());
        assert!(handle_references.references["refs/heads/feature"].is_confirmed());
        assert!(!handle_references.references["refs/tags/v1.0.0"].is_confirmed());
    }

    #[test]
    fn test_get_references_for_updating() {
        let references = create_references();
        let mut handle_references = HandleReferences::new_from_references(&references);

        handle_references.update_local_commit(&vec![Reference::new("abc123", "refs/heads/main").unwrap()]);
        handle_references.update_local_commit(&vec![Reference::new("def456", "refs/heads/feature").unwrap()]);

        // Obtener referencias para actualizar
        let references_for_updating = handle_references.get_references_for_updating().unwrap();

        // Verificar que solo se obtenga la referencia que necesita actualización
        assert_eq!(references_for_updating.len(), 1);
        assert_eq!(references_for_updating[0].get_ref_path(), "refs/tags/v1.0.0");
    }
}