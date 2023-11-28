use std::collections::HashMap;

use crate::{git_transport::references::{Reference, ReferenceType}, util::errors::UtilError};

use super::reference_information::ReferenceInformation;

#[derive(Debug)]
pub struct HandleReferences {
    references: HashMap<String, ReferenceInformation>,
}

impl HandleReferences {
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

    pub fn update_local_commit(&mut self, references: &Vec<Reference>) {
        for reference in references {
            if let Some(reference_status) = self.references.get_mut(reference.get_ref_path()) {
                reference_status.update_local_commit(Some(reference.get_hash().to_string()));
            }
        }
    }

    pub fn get_remote_references(&self) -> Result<Vec<Reference>, UtilError> {
        let mut references: Vec<Reference> = Vec::new();

        for (path, value) in &self.references {
            let reference = Reference::new(value.get_remote_commit().to_string(), path.to_string())?;
            references.push(reference);
        }
        Ok(references)
    }

    pub fn get_local_references(&self) -> Result<Vec<Reference>, UtilError> {
        let mut references: Vec<Reference> = Vec::new();

        for (path, value) in &self.references {
            if let Some(local_commit) = &value.get_local_commit() {
                let reference = Reference::new(local_commit.to_string(), path.to_string())?;
                references.push(reference);
            }
        }
        Ok(references)
    }
}