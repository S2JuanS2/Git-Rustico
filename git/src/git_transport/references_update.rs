use std::io::Write;

use crate::util::{errors::UtilError, validation::is_valid_obj_id, pkt_line::add_length_prefix, connections::send_message};

use super::references::Reference;

pub struct ReferencesUpdate {
    old: String,
    new: String,
    path_refs: String,
}

impl ReferencesUpdate {
    pub fn new(old: String, new: String, path_refs: String) -> ReferencesUpdate {
        ReferencesUpdate {
            old,
            new,
            path_refs,
        }
    }

    pub fn new_from_line(line: &str) -> Result<ReferencesUpdate, UtilError>
    {
        let parts = line.split_ascii_whitespace().collect::<Vec<&str>>();
            if parts.len() != 3 {
                return Err(UtilError::InvalidReferenceUpdateRequest);
            }
            if !is_valid_obj_id(parts[0]) || !is_valid_obj_id(parts[1]) {
                return Err(UtilError::InvalidObjectId);
            }
            let old = parts[0].to_string();
            let new = parts[1].to_string();
            let reference = parts[2].to_string();
            if Reference::is_valid_references_path(&reference) {
                return Err(UtilError::InvalidReferencePath);
            }
            Ok(ReferencesUpdate::new(old, new, reference))
    }
}


pub fn send_status_update_request(writer: &mut dyn Write, reference: &Vec<(String, bool)>) -> Result<(), UtilError> {
    // unpack ok\n
    let signature = "unpack ok\n";
    let message = add_length_prefix(signature, signature.len());
    send_message(writer, &message, UtilError::SendStatusUpdateRequest)?;
    
    for (reference, status) in reference {
        let message = match status {
            true => format!("ok {}\n", reference),
            false => format!("ng {} non-fast-forward\n", reference),
        };
        let message = add_length_prefix(&message, message.len());
        send_message(writer, &message, UtilError::SendStatusUpdateRequest)?;
    }
    Ok(())
}