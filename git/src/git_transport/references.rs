use crate::commands::branch::get_current_branch;
use crate::commands::cat_file::git_cat_file;
use crate::commands::checkout::get_tree_hash;
use crate::errors::GitError;
use crate::git_server::GitServer;
use crate::util::files::{open_file, read_file, read_file_string};
use crate::util::objects::ObjectType;
use crate::{
    consts::{DIRECTORY, FILE, GIT_DIR, HEAD, REFS_REMOTES, REFS_TAGS, REF_HEADS},
    util::{
        connections::send_message, errors::UtilError, pkt_line, validation::join_paths_correctly,
    },
};
use std::{
    fs,
    net::TcpStream,
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ReferenceType {
    Tag,
    Branch,
    Remote,
    Head,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    hash: String,
    ref_path: String,
    reference_type: ReferenceType,
}

impl Reference {
    pub fn new(hash: String, ref_path: String) -> Result<Reference, UtilError> {
        if ref_path == "HEAD" {
            Ok(Reference {
                hash,
                ref_path,
                reference_type: ReferenceType::Head,
            })
        } else if ref_path.starts_with("refs/tags/") {
            Ok(Reference {
                hash,
                ref_path,
                reference_type: ReferenceType::Tag,
            })
        } else if ref_path.starts_with("refs/heads/") {
            Ok(Reference {
                hash,
                ref_path,
                reference_type: ReferenceType::Branch,
            })
        } else if ref_path.starts_with("refs/remotes/") {
            Ok(Reference {
                hash,
                ref_path,
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

    pub fn get_ref_path(&self) -> &String {
        &self.ref_path
    }

    pub fn get_type(&self) -> &ReferenceType {
        &self.reference_type
    }

    pub fn get_name(&self) -> &str {
        let parts: Vec<&str> = self.ref_path.split('/').collect();
        parts.last().map_or("", |&x| x)
    }
}

/// Extrae el contenido de un objeto a partir de su hash
///
/// # Argumentos
///
/// * `hash_object` - Hash del objeto
/// * `directory` - directorio del repositorio
///
/// # Retorna
///
/// Un resultado con el contenido del objeto si la operación es exitosa.
/// En caso de error, retorna un error de tipo UtilError.
fn get_content(directory: &str, hash_object: &str) -> Result<Vec<u8>, UtilError> {
    let path_object = format!(
        "{}/{}/objects/{}/{}",
        directory,
        GIT_DIR,
        &hash_object[..2],
        &hash_object[2..]
    );
    let file_object = open_file(&path_object).expect("Error");
    let content_object = read_file(file_object).expect("Error");

    Ok(content_object)
}

/// Recorre los sub-tree recursivamente y los agrega al vector objects
///
/// # Argumentos
///
/// * `directory` - directorio del repositorio
/// * `tree_hash` - Hash del tree
/// * `objects` - Vector para guardar los objetos a enviar
///
/// # Retorna
///
/// En caso de error, retorna un error de tipo UtilError.
pub fn recovery_tree(
    directory: &str,
    tree_hash: &str,
    objects: &mut Vec<(ObjectType, Vec<u8>)>,
) -> Result<(), GitError> {
    let tree_content = git_cat_file(directory, tree_hash, "-p")?;
    
    for line in tree_content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let mode = parts[1];
        let hash = parts[2];
        if mode == FILE {
            let mut object_blob: (ObjectType, Vec<u8>) = (ObjectType::Blob, Vec::new());
            object_blob.1 = get_content(directory, hash)?;
            objects.push(object_blob);
        } else if mode == DIRECTORY {
            let mut object_tree: (ObjectType, Vec<u8>) = (ObjectType::Tree, Vec::new());
            object_tree.1 = get_content(directory, hash)?;
            objects.push(object_tree);
            recovery_tree(directory, hash, objects)?;
        }
    }
    Ok(())
}

/// Extrae los objetos de un repositorio para guardar los mismos en un vector
///
/// # Argumentos
///
/// * `directory` - directorio del repositorio
/// * `references` - Rama actual del directorio
///
/// # Retorna
///
/// Un vector con el contenido de los objetos si la operación es exitosa.
/// En caso de error, retorna un error de tipo UtilError.
pub fn get_objects(
    directory: &str,
    references: &[Reference],
) -> Result<Vec<(ObjectType, Vec<u8>)>, GitError> {
    let mut objects: Vec<(ObjectType, Vec<u8>)> = vec![];
    for reference in references.iter() {
        let parts: Vec<&str> = reference.get_ref_path().split('/').collect();
        let branch = parts.last().map_or("", |&x| x);
        let branch_current_path = format!("{}/{}/{}/{}", directory, GIT_DIR, REF_HEADS, branch);
        let file_current_branch = open_file(&branch_current_path)?;
        let hash_commit_current_branch = read_file_string(file_current_branch)?;

        let mut object_commit: (ObjectType, Vec<u8>) = (ObjectType::Commit, Vec::new());
        object_commit.1 = get_content(directory, &hash_commit_current_branch)?;

        objects.push(object_commit);

        let content_commit = git_cat_file(directory, &hash_commit_current_branch, "-p")?;
        if let Some(tree_hash) = get_tree_hash(&content_commit) {
            let mut object_tree: (ObjectType, Vec<u8>) = (ObjectType::Tree, Vec::new());
            object_tree.1 = get_content(directory, tree_hash)?;

            objects.push(object_tree);

            recovery_tree(directory, tree_hash, &mut objects)?;
        };
    }
    Ok(objects)
}

/// Extrae la branch actual y el hash del ultimo commit.
///
/// # Argumentos
///
/// * `directory` - directorio del repositorio
///
/// # Retorna
///
/// Una referencia de la rama si la operación es exitosa.
/// En caso de error, retorna un error de tipo UtilError.
pub fn get_ref_name(directory: &str) -> Result<Reference, UtilError> {
    let current_branch = get_current_branch(directory).expect("Error");
    let ref_path = format!("refs/heads/{}", current_branch);
    let branch_current_path = format!("{}/{}/{}/{}", directory, GIT_DIR, REF_HEADS, current_branch);
    if fs::metadata(&branch_current_path).is_err() {
        return Err(UtilError::GenericError);
    }
    let file_current_branch = open_file(&branch_current_path).expect("Error");
    let hash_current_branch = read_file_string(file_current_branch).expect("Error");

    Reference::new(hash_current_branch, ref_path)
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
    stream: &mut TcpStream,
    message: String,
    src_repo: &str,
) -> Result<GitServer, UtilError> {
    send_message(stream, &message, UtilError::ReferenceDiscovey)?;
    let lines = pkt_line::read(stream)?;
    GitServer::new(&lines, src_repo)
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
fn extract_references_from_path(
    path_root: &Path,
    subdirectory: &str,
    signature: &str,
) -> Result<Vec<Reference>, UtilError> {
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
        for entry in entries.flatten() {
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
    if let Some(reference) = trimmed_line.split_once(' ').map(|x| x.1) {
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
fn extract_name_head_from_path(path_git: &str) -> Result<String, UtilError> {
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
fn extract_hash_head_from_path(
    refs: &Vec<Reference>,
    name_head: &str,
) -> Result<String, UtilError> {
    for reference in refs {
        if reference.get_ref_path() == name_head {
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
fn get_reference_head(path_git: &str, refs: &Vec<Reference>) -> Result<Reference, UtilError> {
    let mut name_head = extract_name_head_from_path(path_git)?;
    if let Some('/') = name_head.chars().next() {
        name_head.remove(0);
    }
    let hash_head = extract_hash_head_from_path(refs, &name_head)?;
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
            assert_eq!(reference.get_ref_path(), &"HEAD".to_string());
            assert_eq!(*reference.get_type(), ReferenceType::Head);
        }
    }

    #[test]
    fn test_create_tag_reference() {
        let result = Reference::new("some_hash".to_string(), "refs/tags/version-1.0".to_string());
        assert!(result.is_ok());

        if let Ok(reference) = result {
            assert_eq!(reference.get_ref_path(), &"refs/tags/version-1.0".to_string());
            assert_eq!(*reference.get_type(), ReferenceType::Tag);
        }
    }

    #[test]
    fn test_create_branch_reference() {
        let result = Reference::new("some_hash".to_string(), "refs/heads/main".to_string());
        assert!(result.is_ok());

        if let Ok(reference) = result {
            assert_eq!(reference.get_ref_path(), &"refs/heads/main".to_string());
            assert_eq!(*reference.get_type(), ReferenceType::Branch);
        }
    }

    #[test]
    fn test_create_remote_reference() {
        let result = Reference::new(
            "some_hash".to_string(),
            "refs/remotes/origin/main".to_string(),
        );
        assert!(result.is_ok());

        if let Ok(reference) = result {
            assert_eq!(
                reference.get_ref_path(),
                &"refs/remotes/origin/main".to_string()
            );
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
            ref_path: "refs/heads/main".to_string(),
            reference_type: ReferenceType::Branch,
        };
        assert_eq!(*reference.get_hash(), "some_hash".to_string());
    }

    #[test]
    fn test_get_ref_path() {
        let reference = Reference {
            hash: "some_hash".to_string(),
            ref_path: "refs/tags/version-1.0".to_string(),
            reference_type: ReferenceType::Tag,
        };
        assert_eq!(*reference.get_ref_path(), "refs/tags/version-1.0".to_string());
    }

    #[test]
    fn test_get_type() {
        let reference = Reference {
            hash: "some_hash".to_string(),
            ref_path: "refs/remotes/origin/main".to_string(),
            reference_type: ReferenceType::Remote,
        };
        assert_eq!(*reference.get_type(), ReferenceType::Remote);
    }

    #[test]
    fn test_get_name() {
        // Arrange
        let reference = Reference {
            hash: String::from("abc123"),
            ref_path: String::from("refs/heads/main"),
            reference_type: ReferenceType::Branch,
        };

        // Act
        let name = reference.get_name();

        // Assert
        assert_eq!(name, "main");
    }

    #[test]
    fn test_get_name_with_empty_path() {
        // Arrange
        let reference = Reference {
            hash: String::from("abc123"),
            ref_path: String::from(""),
            reference_type: ReferenceType::Branch,
        };

        // Act
        let name = reference.get_name();

        // Assert
        assert_eq!(name, "");
    }

    #[test]
    fn test_get_name_with_single_component_path() {
        // Arrange
        let reference = Reference {
            hash: String::from("abc123"),
            ref_path: String::from("refs/tags/version1"),
            reference_type: ReferenceType::Tag,
        };

        // Act
        let name = reference.get_name();

        // Assert
        assert_eq!(name, "version1");
    }
}
