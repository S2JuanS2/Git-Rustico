use std::{net::TcpStream, fs, path::Path, io};

use crate::commands::checkout::get_tree_hash;
use crate::errors::GitError;
use crate::util::{errors::UtilError, connections::send_message, pkt_line, validation::join_paths_correctly};
use crate::commands::branch::get_current_branch;
use crate::commands::cat_file::git_cat_file;
use crate::util::files::{open_file, read_file, read_file_string};
use crate::consts::{GIT_DIR, REF_HEADS};
use super::advertised::AdvertisedRefs;


#[derive(Debug, PartialEq, Eq)]
pub enum ReferenceType {
    Tag,
    Branch,
    Remote,
    Head,
}

#[derive(Debug)]
pub struct Reference {
    hash: String,
    refname: String,
    reference_type: ReferenceType,
}

impl Reference {
    pub fn new(hash: String, name: String) -> Result<Reference, UtilError> {
        if name == "HEAD" {
            Ok(Reference {
                hash,
                refname: name,
                reference_type: ReferenceType::Head,
            })
        } else if name.starts_with("refs/tags/") {
            Ok(Reference {
                hash,
                refname: name,
                reference_type: ReferenceType::Tag,
            })
        } else if name.starts_with("refs/heads/") {
            Ok(Reference {
                hash,
                refname: name,
                reference_type: ReferenceType::Branch,
            })
        } else if name.starts_with("refs/remotes/") {
            Ok(Reference {
                hash,
                refname: name,
                reference_type: ReferenceType::Remote,
            })
        } else {
            return Err(UtilError::TypeInvalideference);
        }
    }


    pub fn extract_references_from_git(path: &str) -> Result<Vec<(String, String)>, io::Error> {
        let path = join_paths_correctly(path, ".git");
        let mut references: Vec<(String, String)> = Vec::new();
        let refs = Path::new(&path).join("refs");
        let refs_branch = refs.join("heads");
        let _refs_tag = refs.join("tags");
        let _refs_remote = refs.join("remotes");
        
        for entry in fs::read_dir(refs_branch)? {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                if !entry_path.is_file() {
                    continue;
                }
                let name = entry_path.as_path().display().to_string();
                if let Ok(hash) = fs::read_to_string(entry_path) {
                    references.push((hash.trim().to_string(), name));
                }
            }
        }
        Ok(references)
    }

    pub fn get_hash(&self) -> &String {
        &self.hash
    }

    pub fn get_name(&self) -> &String {
        &self.refname
    }

    pub fn get_type(&self) -> &ReferenceType {
        &self.reference_type
    }
}

fn get_content(directory: &str, hash_object: &str) -> Result<Vec<u8>, UtilError> {

    let path_object = format!("{}/{}/objects/{}/{}", directory, GIT_DIR, &hash_object[..2], &hash_object[2..]);
    let file_object = open_file(&path_object).expect("Error");
    let content_object = read_file(file_object).expect("Error");

    Ok(content_object)
}

fn get_objects(directory: &str) -> Result<Vec<Vec<u8>>, GitError> {

    let mut objects: Vec<Vec<u8>> = vec![];

    let current_branch = get_current_branch(directory)?;
    let branch_current_path = format!("{}/{}/{}/{}", directory, GIT_DIR, REF_HEADS, current_branch);
    let file_current_branch = open_file(&branch_current_path)?;
    let hash_commit_current_branch = read_file_string(file_current_branch)?;

    objects.push(get_content(directory, &hash_commit_current_branch)?);

    let content_commit = git_cat_file(directory, &hash_commit_current_branch, "-p")?;
    let tree_hash = get_tree_hash(&content_commit).expect("Error");

    objects.push(get_content(directory, tree_hash)?);
    
    let tree_content = git_cat_file(directory, tree_hash, "-p")?;

    for line in tree_content.lines() {

        let parts: Vec<&str> = line.split_whitespace().collect();
        let hash_blob = parts[2];
        objects.push(get_content(directory, hash_blob)?);
    }
 
    Ok(objects)
}

fn get_ref_name(directory: &str) -> Result<Reference, UtilError> {
    let current_branch = get_current_branch(directory).expect("Error");
    let refname = format!("refs/heads/{}", current_branch);
    let branch_current_path = format!("{}/{}/{}/{}", directory, GIT_DIR, REF_HEADS, current_branch);
    if fs::metadata(&branch_current_path).is_err() {
        return Err(UtilError::GenericError);
    }
    let file_current_branch = open_file(&branch_current_path).expect("Error");
    let hash_current_branch = read_file_string(file_current_branch).expect("Error");

    Reference::new(hash_current_branch, refname)
}

/// Realiza un proceso de descubrimiento de referencias (refs) enviando un mensaje al servidor
/// a través del socket proporcionado, y luego procesa las líneas recibidas para clasificarlas
/// en una lista de AdvertisedRefLine.
///
/// # Argumentos
/// - `socket`: Un TcpStream que representa la conexión con el servidor.
/// - `message`: Un mensaje que se enviará al servidor.
///
/// # Retorno
/// Un Result que contiene un vector de AdvertisedRefLine si la operación fue exitosa,
/// o un error de UtilError en caso contrario.
pub fn reference_discovery(
    socket: &mut TcpStream,
    message: String,
) -> Result<AdvertisedRefs, UtilError> {
    send_message(socket, message, UtilError::ReferenceDiscovey)?;
    let lines = pkt_line::read(socket)?;
    println!("lines: {:?}", lines);
    AdvertisedRefs::new(&lines)
}

// A mejorar
// El packet-ref deberia eliminar esto
// pub fn list_references(repo_path: &str) -> Result<Vec<String>, UtilError> {
//     let mut references: Vec<String> = Vec::new();

//     let refs_dir = format!("{}/.git/refs", repo_path);

//     if let Ok(entries) = fs::read_dir(refs_dir) {
//         for entry in entries {
//             if let Ok(entry) = entry {
//                 if let Some(file_name) = entry.file_name().to_str() {
//                     if file_name.starts_with("heads/") || file_name.starts_with("tags/") {
//                         references.push(file_name.to_string());
//                     }
//                 }
//             }
//         }
//     }

//     Ok(references)
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_head_reference() {
        let result = Reference::new("some_hash".to_string(), "HEAD".to_string());
        assert!(result.is_ok());

        if let Ok(reference) = result {
            assert_eq!(reference.get_name(), &"HEAD".to_string());
            assert_eq!(*reference.get_type(), ReferenceType::Head);
        }
    }

    #[test]
    fn test_create_tag_reference() {
        let result = Reference::new("some_hash".to_string(), "refs/tags/version-1.0".to_string());
        assert!(result.is_ok());

        if let Ok(reference) = result {
            assert_eq!(reference.get_name(), &"refs/tags/version-1.0".to_string());
            assert_eq!(*reference.get_type(), ReferenceType::Tag);
        }
    }

    #[test]
    fn test_create_branch_reference() {
        let result = Reference::new("some_hash".to_string(), "refs/heads/main".to_string());
        assert!(result.is_ok());

        if let Ok(reference) = result {
            assert_eq!(reference.get_name(), &"refs/heads/main".to_string());
            assert_eq!(*reference.get_type(), ReferenceType::Branch);
        }
    }

    #[test]
    fn test_create_remote_reference() {
        let result = Reference::new("some_hash".to_string(), "refs/remotes/origin/main".to_string());
        assert!(result.is_ok());

        if let Ok(reference) = result {
            assert_eq!(reference.get_name(), &"refs/remotes/origin/main".to_string());
            assert_eq!(*reference.get_type(), ReferenceType::Remote);
        }
    }

    #[test]
    fn test_create_invalid_reference() {
        let result = Reference::new("some_hash".to_string(), "invalid_reference".to_string());
        assert!(result.is_err());
    }


    #[test]
    fn test_get_hash() {
        let reference = Reference {
            hash: "some_hash".to_string(),
            refname: "refs/heads/main".to_string(),
            reference_type: ReferenceType::Branch,
        };
        assert_eq!(*reference.get_hash(), "some_hash".to_string());
    }

    #[test]
    fn test_get_name() {
        let reference = Reference {
            hash: "some_hash".to_string(),
            refname: "refs/tags/version-1.0".to_string(),
            reference_type: ReferenceType::Tag,
        };
        assert_eq!(*reference.get_name(), "refs/tags/version-1.0".to_string());
    }

    #[test]
    fn test_get_type() {
        let reference = Reference {
            hash: "some_hash".to_string(),
            refname: "refs/remotes/origin/main".to_string(),
            reference_type: ReferenceType::Remote,
        };
        assert_eq!(*reference.get_type(), ReferenceType::Remote);
    }

    #[test]
    fn test(){
        get_objects("Repository").expect("Error");
    }
}