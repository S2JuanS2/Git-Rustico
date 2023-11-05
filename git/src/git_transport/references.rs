use std::{net::TcpStream, fs, path::{Path, PathBuf}};

use crate::util::{errors::UtilError, connections::send_message, pkt_line, validation::join_paths_correctly};

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


    pub fn extract_references_from_git(root: &str) -> Result<Vec<Reference>, UtilError> {
        println!("extract_references_from_git");
        let path_git = join_paths_correctly(root, ".git");
        
        let path = Path::new(&path_git).join("refs");
        let refs_branch = extract_references_from_path(Path::new(path.as_os_str()).join("heads"), "refs/heads")?;
        let refs_tag = extract_references_from_path(Path::new(path.as_os_str()).join("tags"), "refs/tags")?;
        let refs_remote = extract_references_from_path(Path::new(path.as_os_str()).join("remotes"), "refs/remotes")?;
        
        let mut refs = Vec::new();
        refs.extend(refs_branch);
        refs.extend(refs_tag);
        refs.extend(refs_remote);

        println!("Buscare Head");
        let path_head = Path::new(&path_git).join("HEAD");
        println!("Path head: {:?}", path_head);
        if let Ok(hash) = fs::read_to_string(path_head)
        {
            let head = match extract_reference_head(&hash)
            {
                Some(h) => h,
                None => return Ok(refs),
            };
            println!("HEAD - hash: {}", head);
        }
        Ok(refs)
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


fn extract_references_from_path(path_root: PathBuf, path_relative: &str) -> Result<Vec<Reference>, UtilError>
{
    let mut references = Vec::new();
    let names_refs = get_files_in_directory(&path_root);
    for name in names_refs {
        let path = Path::new(&path_root).join(&name);
        if let Ok(hash) = fs::read_to_string(path) {
            let name_ref = format!("{}/{}", path_relative, name);
            let refs = Reference::new(hash.trim().to_string(), name_ref)?;
            println!("Refs: {:?}", refs);
            references.push(refs);
        }
    }
    Ok(references)
}

fn get_files_in_directory(directory_path: &PathBuf) -> Vec<String> {
    let mut files: Vec<String> = Vec::new();

    if let Ok(entries) = fs::read_dir(directory_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    if let Some(file_name) = path.file_name() {
                        if let Some(name) = file_name.to_str() {
                            files.push(name.to_string());
                        }
                    }
                }
            }
        }
    }

    files
}

fn extract_reference_head(line: &str) -> Option<&str> {
    let trimmed_line = line.trim();
    if let Some(reference) = trimmed_line.splitn(2, ' ').nth(1) {
        Some(reference)
    } else {
        None
    }
}

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
}