use std::{net::TcpStream, fs, path::{Path, PathBuf}};
use crate::commands::checkout::get_tree_hash;
use crate::errors::GitError;
use crate::commands::branch::get_current_branch;
use crate::commands::cat_file::git_cat_file;
use crate::util::files::{open_file, read_file, read_file_string};
use crate::consts::{GIT_DIR, REF_HEADS};

use crate::{util::{errors::UtilError, connections::send_message, pkt_line, validation::join_paths_correctly}, consts::{GIT_DIR, REF_HEADS, REFS_REMOTES, REFS_TAGS, HEAD}};

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


    /// Extrae las referencias de un repositorio Git.
    ///
    /// # Argumentos
    ///
    /// * `root` - Ruta al directorio raíz del repositorio Git.
    ///
    /// # Retorna
    ///
    /// Un resultado que contiene un vector de Referencias si la operación es exitosa.
    /// En caso de error, retorna un error de tipo UtilError.
    pub fn extract_references_from_git(root: &str) -> Result<Vec<Reference>, UtilError> {
        println!("extract_references_from_git");
        let path_git = join_paths_correctly(root, GIT_DIR);
        
        let path = Path::new(&path_git).join("refs");
        let refs_branch = extract_references_from_path(&path, "heads", REF_HEADS)?;
        let refs_tag = extract_references_from_path(&path, "tags", REFS_TAGS)?;
        let refs_remote = extract_references_from_path(&path, "remotes", REFS_REMOTES)?;
        
        let mut refs = Vec::new();
        refs.extend(refs_branch);
        refs.extend(refs_tag);
        refs.extend(refs_remote);

        let head = get_reference_head(&path_git, &refs)?;
        refs.insert(0, head);
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


/// Extrae referencias de un subdirectorio de un directorio base, creando un vector de Referencias.
///
/// # Argumentos
///
/// * `path_root` - Ruta base del directorio.
/// * `subdirectory` - Subdirectorio del que se extraerán las referencias.
/// * `signature` - Firma o identificador de las referencias.
///
/// # Retorna
///
/// Un resultado que contiene un vector de Referencias si la operación es exitosa.
/// En caso de error, retorna un error de tipo UtilError.
/// 
fn extract_references_from_path(path_root: &PathBuf, subdirectory: &str, signature: &str) -> Result<Vec<Reference>, UtilError>
{
    let new_root = Path::new(path_root.as_os_str()).join(subdirectory);
    let mut references = Vec::new();
    let names_refs = get_files_in_directory(&new_root);
    for name in names_refs {
        let path = Path::new(&new_root).join(&name);
        if let Ok(hash) = fs::read_to_string(path) {
            let name_ref = format!("{}/{}", signature, name);
            let refs = Reference::new(hash.trim().to_string(), name_ref)?;
            println!("Refs: {:?}", refs);
            references.push(refs);
        }
    }
    Ok(references)
}

/// Obtiene los nombres de archivo dentro de un directorio.
///
/// # Argumentos
///
/// * `directory_path` - Ruta del directorio del que se desean obtener los nombres de archivo.
///
/// # Retorna
///
/// Un vector de cadenas que contiene los nombres de archivo del directorio especificado.
/// 
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

/// Extrae la referencia HEAD de una línea dada.
///
/// # Argumentos
///
/// * `line` - Cadena que contiene la referencia HEAD.
///
/// # Retorna
///
/// Devuelve un resultado que contiene la cadena de la referencia HEAD si la operación es exitosa.
/// En caso de un formato inválido, retorna un error de tipo UtilError.
/// 
fn extract_reference_head(line: &str) -> Result<String, UtilError> {
    let trimmed_line = line.trim();
    if let Some(reference) = trimmed_line.splitn(2, ' ').nth(1) {
        Ok(reference.to_string())
    } else {
        Err(UtilError::InvalidHeadReferenceFormat)
    }
}

/// Extrae el nombre de la referencia HEAD del archivo 'HEAD' en el directorio '.git'.
///
/// # Argumentos
///
/// * `path_git` - Ruta al directorio '.git'.
///
/// # Retorna
///
/// Devuelve un resultado que contiene el nombre de la referencia HEAD si la operación es exitosa.
/// En caso de que no se encuentre el archivo 'HEAD', retorna un error de tipo UtilError.
/// 
fn extract_name_head_from_path(path_git: &str) -> Result<String, UtilError>
{
    let path = Path::new(&path_git).join("HEAD");
    if let Ok(line) = fs::read_to_string(path) {
        let refs = extract_reference_head(&line)?;
        return Ok(refs);
    }
    Err(UtilError::HeadFolderNotFound)
}

/// Extrae el hash de la referencia HEAD a partir de un vector de referencias y el nombre de la referencia.
///
/// # Argumentos
///
/// * `refs` - Vector de referencias.
/// * `name_head` - Nombre de la referencia HEAD.
///
/// # Retorna
///
/// Devuelve un resultado que contiene el hash de la referencia HEAD si la operación es exitosa.
/// En caso de que no se encuentre el hash correspondiente a la referencia HEAD, retorna un error de tipo UtilError.
/// 
fn extract_hash_head_from_path(refs: &Vec<Reference>, name_head: &str) -> Result<String, UtilError>
{
    for reference in refs {
        if reference.get_name() == name_head {
            return Ok(reference.get_hash().to_string());
        }
    }
    Err(UtilError::HeadHashNotFound)
}

/// Obtiene la referencia HEAD a partir de la ruta al directorio '.git' y un vector de referencias.
///
/// # Argumentos
///
/// * `path_git` - Ruta al directorio '.git'.
/// * `refs` - Vector de referencias.
///
/// # Retorna
///
/// Devuelve un resultado que contiene la referencia HEAD si la operación es exitosa.
/// En caso de fallo al extraer la referencia HEAD, retorna un error de tipo UtilError.
/// 
fn get_reference_head(path_git: &str, refs: &Vec<Reference>) -> Result<Reference, UtilError>
{
    let name_head = extract_name_head_from_path(path_git)?;
    let hash_head = extract_hash_head_from_path(&refs, &name_head)?;
    Reference::new(hash_head, HEAD.to_string())
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

    #[test]
    fn test(){
        get_objects("Repository").expect("Error");
    }
}