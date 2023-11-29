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
    pub fn confirm_local_references(&mut self, local_commits: &Vec<String>)
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


}