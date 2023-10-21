use crate::errors::GitError;

use super::validation::is_valid_obj_id;
use std::{fmt, vec};
#[derive(Debug, Clone)]
pub enum AdvertisedRefs {
    Version(u8),
    Capabilities(Vec<String>), // a implementar
    Ref{obj_id: String, ref_name: String},
    Shallow{obj_id: String},
}

impl fmt::Display for AdvertisedRefs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AdvertisedRefs::Version(version) => write!(f, "Version: {}", version),
            AdvertisedRefs::Ref{obj_id, ref_name} => write!(f, "Ref: (obj: {}, name: {})", obj_id, ref_name),
            AdvertisedRefs::Shallow{obj_id} => write!(f, "Shallow: {}", obj_id),
            AdvertisedRefs::Capabilities(capabilities) => write!(f, "Capabilities: {:?}", capabilities),
        }
    }
}

impl AdvertisedRefs
{
    pub fn classify_vec(content: &Vec<Vec<u8>>) -> Result<Vec<AdvertisedRefs>, GitError> {
        let mut result: Vec<AdvertisedRefs> = Vec::new();
        // let mut lines = content.split(|&c| c == b'\n');
        for c in content {
            if let Ok(line_str) = std::str::from_utf8(c) {
                if let Ok(refs) = AdvertisedRefs::classify_server_refs(line_str) {
                    result.extend(refs);
                }
            }
        }
        Ok(result)
    }

    fn create_version(version: &str) -> Result<Vec<AdvertisedRefs>, GitError> {
        let version_number = version[..].parse::<u8>();
        match version_number.unwrap() {
            1 => Ok(vec![AdvertisedRefs::Version(1)]),
            2 => Ok(vec![AdvertisedRefs::Version(2)]),
            _ => Err(GitError::InvalidVersionNumberError),
        }
    }

    fn create_shallow(obj_id: &str) -> Result<Vec<AdvertisedRefs>, GitError> {
        if is_valid_obj_id(obj_id) == false {
            return Err(GitError::InvalidObjectIdError);
        }
        Ok(vec![AdvertisedRefs::Shallow{obj_id: obj_id.to_string()}])
    }

    fn create_ref(input: &str) -> Result<Vec<AdvertisedRefs>, GitError> {
        
        if !contains_capacity_list(input)
        {
            return _create_ref(input);
        }

        let parts:Vec<&str> = input.split('\0').collect();
        if parts.len() != 2 {
            return Err(GitError::InvalidObjectIdError);
        }

        let mut vec: Vec<AdvertisedRefs> = _create_ref(parts[0])?;
        vec.insert(0, extract_capabilities(parts[1]));
        Ok(vec)
    }

    fn classify_server_refs(input: &str) -> Result<Vec<AdvertisedRefs>, GitError> {
        // print!("Info: {}", info);
        let parts: Vec<&str> = input.split_whitespace().collect();
    
        // Verificar si el primer elemento es una versión válida
        if parts[0] == "version" {
            return AdvertisedRefs::create_version(parts[1]);
        }
        // Verificar si el primer elemento es "shallow"
        if parts[0] == "shallow" {
            return AdvertisedRefs::create_shallow(parts[1]);
        }
        
        // Verificar si el segundo elemento parece ser una referencia
        if parts[1].starts_with("refs/") || parts[1].starts_with("HEAD"){
            return AdvertisedRefs::create_ref(input);
        }
        Err(GitError::InvalidServerReferenceError)
    }

}

fn extract_capabilities(input: &str) -> AdvertisedRefs {
    let mut capabilities: Vec<String> = Vec::new();
    capabilities.extend(input.split_whitespace().map(String::from));
    return AdvertisedRefs::Capabilities(capabilities);
}

fn contains_capacity_list(input: &str) -> bool {
    input.contains('\0')
}

fn _create_ref(input: &str) -> Result<Vec<AdvertisedRefs>, GitError>
{
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() != 2 {
        return Err(GitError::InvalidServerReferenceError);
    }
    if is_valid_obj_id(parts[0]) == false {
        return Err(GitError::InvalidObjectIdError);
    }
    Ok(vec![AdvertisedRefs::Ref{obj_id: parts[0].to_string(), ref_name: parts[1].to_string()}])
}