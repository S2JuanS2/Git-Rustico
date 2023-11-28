use std::collections::HashMap;

use crate::git_transport::references::{Reference, ReferenceType};

use super::reference_information::ReferenceInformation;

#[derive(Debug)]
pub struct ClientReferences {
    references: HashMap<String, ReferenceInformation>,
}

impl ClientReferences {
    pub fn new_from_references(references: &Vec<Reference>) -> Self {
        let mut references_map: HashMap<String, ReferenceInformation> = HashMap::new();

        for reference in references {
            // Ignore Head
            if reference.get_type() == &ReferenceType::Tag {
                continue;
            }

            references_map.insert(
                reference.get_ref_path().to_string(),
                ReferenceInformation::new(reference.get_hash(), None),
            );
        }

        ClientReferences {
            references: references_map,
        }
    }

    pub fn update_current_commit(&mut self, references: &Vec<Reference>) {
        for reference in references {
            if let Some(reference_status) = self.references.get_mut(reference.get_ref_path()) {
                reference_status.update_current_commit(Some(reference.get_hash().to_string()));
            }
        }
    }
}