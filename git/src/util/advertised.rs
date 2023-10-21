use crate::errors::GitError;

use super::validation::is_valid_obj_id;
use std::fmt;
#[derive(Debug, Clone)]
pub enum AdvertisedRefs {
    Version(u8),
    // Capabilities(Vec<String>), // a implementar
    Ref{obj_id: String, ref_name: String},
    Shallows{obj_id: String},
}

impl fmt::Display for AdvertisedRefs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AdvertisedRefs::Version(version) => write!(f, "Version: {}", version),
            AdvertisedRefs::Ref{obj_id, ref_name} => write!(f, "Ref: (obj: {}, name: {})", obj_id, ref_name),
            AdvertisedRefs::Shallows{obj_id} => write!(f, "Shallow: {}", obj_id),
            
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
        Ok(vec![AdvertisedRefs::Shallows{obj_id: obj_id.to_string()}])
    }

    fn create_ref(obj_id: &str, ref_name: &str) -> Result<Vec<AdvertisedRefs>, GitError> {
        if is_valid_obj_id(obj_id) == false {
            return Err(GitError::InvalidObjectIdError);
        }
        Ok(vec![AdvertisedRefs::Ref{obj_id: obj_id.to_string(), ref_name: ref_name.to_string()}])
    }

    fn classify_server_refs(info: &str) -> Result<Vec<AdvertisedRefs>, GitError> {
        // print!("Info: {}", info);
        let parts: Vec<&str> = info.split_whitespace().collect();
    
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
            return AdvertisedRefs::create_ref(parts[0], parts[1]);
        }
    
    
        Err(GitError::InvalidServerReferenceError)
    }

}

// d8e5f4121f852fa9612d145675cfb2ccac68b150 HEAD\0multi_ack thin-pack side-band side-band-64k ofs-delta shallow deepen-since deepen-not deepen-relative no-progress include-tag multi_ack_detailed symref=HEAD:refs/heads/main object-format=sha1 agent=git/2.42.0
// fn extract_capabilities(pkt_line: &str) -> (Vec<u8>, Vec<String>) {
//     let mut capabilities: Vec<String> = Vec::new();
//     let reference = pkt_line;
//     let pkt_line_str = String::from_utf8_lossy(pkt_line);

//     if let Some(pos) = pkt_line.find('\0') {
//         // Extract the part of the line after the NUL character, which contains capabilities.
//         let capabilities_str = &pkt_line[pos + 1..];
//         capabilities.extend(capabilities_str.split_whitespace().map(String::from));
//     }

//     (reference.to_vec(), capabilities)
// }