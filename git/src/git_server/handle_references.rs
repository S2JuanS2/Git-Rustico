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
                reference_status.update_local_commit(Some(reference.get_hash().trim().to_string()));
            }
        }
    }

    pub fn get_remote_references(&self) -> Result<Vec<Reference>, UtilError> {
        let mut references: Vec<Reference> = Vec::new();

        for (path, value) in &self.references {
            let reference = Reference::new(value.get_remote_commit(), path)?;
            references.push(reference);
        }
        Ok(references)
    }

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

    pub fn confirm_local_references(&mut self, local_commits: &Vec<String>)
    {
        println!("Confirming local references");
        println!("Local commits: {:?}", local_commits);
        println!("Self: {:?}", self);
        for value in self.references.values_mut() {
            if let Some(local_commit) = value.get_local_commit() {
                if local_commits.contains(&local_commit.to_string()) {
                    value.confirm_local_commit();
                }
            }
        }
    }

    pub fn get_updated_references(&self) -> Result<Vec<Reference>, UtilError> {
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