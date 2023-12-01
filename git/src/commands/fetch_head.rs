use std::{io::{self, Write, BufRead}, fs, fmt};

use crate::git_transport::references::Reference;

use super::errors::CommandsError;


/// Enum que representa las posibles etiquetas para FetchHeadEntry.
#[derive(Debug, PartialEq)]
pub enum Label {
    NotForMerge,
    Merge,
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Label::NotForMerge => write!(f, "not-for-merge"),
            Label::Merge => write!(f, ""),
        }
    }
}

/// Struct que representa una entrada en el archivo FETCH_HEAD.
#[derive(Debug, PartialEq)]
pub struct FetchHeadEntry {
    commit_hash: String,
    branch_name: String,
    label: Label,
    remote_repo: String,
}

impl fmt::Display for FetchHeadEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}\t{}\tbranch '{}' of github.com:{}", self.commit_hash, self.label, self.branch_name, self.remote_repo)
    }
}

impl FetchHeadEntry {
    /// Crea una nueva entrada FETCH_HEAD.
    ///
    /// # Arguments
    ///
    /// * `commit_hash` - Hash del commit.
    /// * `branch_name` - Nombre de la rama.
    /// * `label` - Etiqueta que indica si la entrada está marcada como "not-for-merge" o "merge".
    /// * `remote_repo` - Path del repositorio remoto asociado.
    ///
    /// # Returns
    ///
    /// Retorna un resultado que contiene la nueva entrada FETCH_HEAD o un error.
    ///
    /// # Errors
    ///
    /// Retorna un error si la etiqueta proporcionada no es válida.
    pub fn new(commit_hash: String, branch_name: String, label: String, remote_repo: String) -> Result<Self, CommandsError> {
        let label = match label.as_str() {
            "not-for-merge" => Label::NotForMerge,
            "" => Label::Merge,
            _ => Err(CommandsError::InvalidFetchHeadEntry)?,
        };
        Ok(FetchHeadEntry {
            commit_hash,
            branch_name,
            label,
            remote_repo,
        })
    }

    /// Crea una nueva entrada FETCH_HEAD a partir de una línea del archivo FETCH_HEAD.
    ///
    /// # Arguments
    ///
    /// * `line` - Línea del archivo FETCH_HEAD.
    ///
    /// # Returns
    ///
    /// Retorna un resultado que contiene la nueva entrada FETCH_HEAD o un error.
    ///
    /// # Errors
    ///
    /// Retorna un error si la línea proporcionada no tiene el formato correcto.
    pub fn new_from_line(line: &str) -> Result<Self, CommandsError> {
        let parts: Vec<&str> = line.split('\t').collect();

        if parts.len() != 3 {
            return Err(CommandsError::InvalidFetchHeadEntry);
        }
        let hash = parts[0].to_string();
        let mode_merge = parts[1].to_string();
        let branch_info = parts[2].to_string();
        let (name, remote) = match extract_branch_info(&branch_info)
        {
            Ok((branch_name, remote_repo)) => (branch_name, remote_repo),
            Err(_) => return Err(CommandsError::InvalidFetchHeadEntry),
        };

        FetchHeadEntry::new(hash, name, mode_merge, remote)
    }
}

#[derive(Debug, PartialEq)]
pub struct FetchHead {
    entries: Vec<FetchHeadEntry>,
}

impl FetchHead {

    /// Crea un nuevo archivo FETCH_HEAD a partir de referencias locales.
    ///
    /// # Arguments
    ///
    /// * `references` - Referencias locales en forma de tuplas (nombre, hash).
    /// * `remote_repo` - Path del repositorio remoto.
    ///
    /// # Returns
    ///
    /// Retorna un resultado que contiene el nuevo archivo FETCH_HEAD o un error.
    ///
    /// # Errors
    ///
    /// Retorna un error si hay algún problema al crear las entradas FETCH_HEAD.
    /// 
    pub fn new(
        references: &Vec<Reference>,
        remote_repo: &str
    ) -> Result<FetchHead, CommandsError> {
        let mut entries = Vec::new();
        for reference in references {
            let commit_hash = reference.get_hash().to_string();
            let branch_name = reference.get_name().to_string();
            let entry = FetchHeadEntry::new(commit_hash, branch_name, Label::Merge.to_string(), remote_repo.to_string());
            entries.push(entry?);
        }
        Ok(FetchHead {
            entries,
        })
    }

     /// Escribe el contenido de FetchHead en el archivo FETCH_HEAD.
    ///
    /// # Argumentos
    ///
    /// * `repo_local` - Ruta local del repositorio.
    ///
    /// # Retorno
    ///
    /// Resultado que indica si la operación de escritura fue exitosa o si se produjo un error.
    ///
    /// # Errores
    ///
    /// Devuelve un error de tipo CommandsError si ocurre algún problema durante la escritura.
    /// 
    pub fn write(&self, repo_local: &str) -> Result<(), CommandsError> {
        let repo = format!("{}/.git", repo_local);
        let fetch_head_path = format!("{}/FETCH_HEAD", repo);
        match self._write(&fetch_head_path)
        {
            Ok(_) => Ok(()),
            Err(_) => Err(CommandsError::WriteFetchHEAD),
        }
    }

    // Método auxiliar que realiza la escritura real en el archivo FETCH_HEAD.
    fn _write(&self, fetch_head_path: &str) -> io::Result<()>
    {
        let mut file = fs::File::create(fetch_head_path)?;
        for entry in &self.entries {
            let line = format!("{}", entry);
            file.write_all(line.as_bytes())?;
        }
        file.flush()?;
        Ok(())
    }

    /// Crea una nueva instancia de `FetchHead` leyendo el contenido del archivo FETCH_HEAD en el repositorio.
    ///
    /// # Arguments
    ///
    /// * `repo_path` - Ruta al directorio del repositorio local.
    ///
    /// # Returns
    ///
    /// Retorna un resultado que contiene la estructura `FetchHead` o un error si no se puede leer el archivo.
    ///
    /// # Errors
    ///
    /// Retorna un error si el archivo FETCH_HEAD no se encuentra o si hay problemas al leer su contenido.
    pub fn new_from_file(repo_path: &str) -> Result<FetchHead, CommandsError> {
        let repo = format!("{}/.git/FETCH_HEAD", repo_path);
        _read_fetch_head(&repo)
    }

    pub fn references_needs_update(&self, branch: &str) -> bool {
        for entry in &self.entries {
            if entry.branch_name == branch && entry.label == Label::Merge {
                return true;
            }
        }
        false
    }

    pub fn delete_references(&mut self, branch: &str) -> Result<(), CommandsError> {
        for i in 0..self.entries.len() {
            if self.entries[i].branch_name == branch && self.entries[i].label == Label::Merge {
                self.entries.remove(i);
                return Ok(());
            }
        }
        Err(CommandsError::DeleteReferenceFetchHead)
    }

    pub fn get_references(&self, branch: &str) -> Result<Reference, CommandsError> {
        for entry in &self.entries {
            if entry.branch_name == branch && entry.label == Label::Merge {
                return Ok(Reference::new(&entry.commit_hash, &entry.branch_name)?);
            }
        }
        Err(CommandsError::ReferenceNotFound)
    }

    // pub fn update(&mut self, references: &Vec<Reference>, remote_repo: &str) -> Result<(), CommandsError> {
    //     for reference in references {
    //         let commit_hash = reference.get_hash().to_string();
    //         let branch_name = reference.get_name().to_string();
    //         let entry = FetchHeadEntry::new(commit_hash, branch_name, Label::Merge.to_string(), remote_repo.to_string());
    //         self.entries.push(entry?);
    //     }
    //     Ok(())
    // }
}



/// Extrae la información de la rama y el repositorio remoto desde una cadena de información de la rama.
///
/// # Arguments
///
/// * `branch_info` - Información de la rama en el formato específico del archivo FETCH_HEAD.
///
/// # Returns
///
/// Retorna un resultado que contiene una tupla con el nombre de la rama y la URL del repositorio remoto o un error.
///
/// # Errors
///
/// Retorna un error si la cadena no sigue el formato esperado.
/// 
fn extract_branch_info(branch_info: &str) -> Result<(String, String), CommandsError> {
    let prefix = "branch '";
    let suffix = "' of github.com:";

    if let Some(start_pos) = branch_info.find(prefix) {
        let start_pos = start_pos + prefix.len();
        if let Some(end_pos) = branch_info[start_pos..].find(suffix) {
            let branch_name = &branch_info[start_pos..start_pos + end_pos];
            let rest = &branch_info[start_pos + end_pos + suffix.len()..];
            return Ok((branch_name.to_string(), rest.to_string()));
        }
    }
    Err(CommandsError::InvalidFetchHeadEntry)
}

/// Lee el contenido del archivo FETCH_HEAD y crea una instancia de `FetchHead` con las entradas correspondientes.
///
/// # Arguments
///
/// * `path` - Ruta al archivo FETCH_HEAD en el repositorio.
///
/// # Returns
///
/// Retorna un resultado que contiene la estructura `FetchHead` o un error si no se puede leer el archivo.
///
/// # Errors
///
/// Retorna un error si el archivo FETCH_HEAD no se encuentra o si hay problemas al leer su contenido.
/// 
fn _read_fetch_head(fetch_head_path: &str) -> Result<FetchHead, CommandsError> {
    let file = match fs::File::open(fetch_head_path)
    {
        Ok(file) => file,
        Err(_) => return Err(CommandsError::FetchHeadFileNotFound),
    };
    let mut entries = Vec::new();
    for line in io::BufReader::new(file).lines() {
        let line = match line
        {
            Ok(line) => line,
            Err(_) => return Err(CommandsError::ReadFetchHEAD),
        };
        let entrie = FetchHeadEntry::new_from_line(&line)?;
        entries.push(entrie);
    }
    Ok(FetchHead {
        entries,
    })
}


// /// Obtiene las referencias del archivo FETCH_HEAD que tienen el modo de fusión "not-for-merge".
// ///
// /// # Argumentos
// ///
// /// * `repo_path` - Ruta del repositorio.
// ///
// /// # Errores
// ///
// /// Devuelve un error de tipo `CommandsError` si no puede leer o interpretar el contenido del archivo FETCH_HEAD.
// ///
// // pub fn get_references_not_for_merge(repo_path: &str) -> Result<Vec<(String, String)>, CommandsError> {
// //     let references = read_fetch_head(repo_path)?;
// //     let mut filter = Vec::new();
// //     for (hash, mode_merge, branch_github) in references {
// //         if mode_merge == "not-for-merge" {
// //             filter.push((hash, branch_github));
// //         }
// //     }
// //     Ok(filter)
// // }


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_branch_info_valid() {
        let branch_info = "branch 'my-branch' of github.com:example/repo";
        let result = extract_branch_info(branch_info);
        assert!(result.is_ok());

        let (branch_name, rest) = result.unwrap();
        assert_eq!(branch_name, "my-branch");
        assert_eq!(rest, "example/repo");
    }

    #[test]
    fn test_extract_branch_info_invalid() {
        let branch_info = "invalid_format";
        let result = extract_branch_info(branch_info);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CommandsError::InvalidFetchHeadEntry);
    }

    #[test]
    fn test_fetch_head_entry_new_from_line_not_for_merge() {
        let line = "d3214e19f4736504392664d579ce1ef2d15b5581	not-for-merge	branch 'main' of github.com:example/repo";
        let result = FetchHeadEntry::new_from_line(line);
        print!("{:?}", result);
        assert!(result.is_ok());

        let entry = result.unwrap();
        assert_eq!(entry.commit_hash, "d3214e19f4736504392664d579ce1ef2d15b5581");
        assert_eq!(entry.branch_name, "main");
        assert_eq!(entry.label, Label::NotForMerge);
        assert_eq!(entry.remote_repo, "example/repo");
    }

    #[test]
    fn test_fetch_head_entry_new_from_line_merge() {
        let line = "d3214e19f4736504392664d579ce1ef2d15b5581		branch 'main' of github.com:example/repo";
        let result = FetchHeadEntry::new_from_line(line);
        assert!(result.is_ok());

        let entry = result.unwrap();
        assert_eq!(entry.commit_hash, "d3214e19f4736504392664d579ce1ef2d15b5581");
        assert_eq!(entry.branch_name, "main");
        assert_eq!(entry.label, Label::Merge);
        assert_eq!(entry.remote_repo, "example/repo");
    }

    #[test]
    fn test_fetch_head_entry_new_from_line_invalid() {
        let line = "invalid_format";
        let result = FetchHeadEntry::new_from_line(line);
        assert!(result.is_err());
        assert!(result.is_err());
    }

    #[test]
    fn test_new_fetch_head_with_references() {
        // Simula algunas referencias locales para la prueba
        let references = vec![
            Reference::new("93455fe53543e1dcca9533dd51d5b83656a6432c", "refs/heads/branch1").unwrap(),
            Reference::new("56620fe39508e1dcca4873dd51d5b83656a9418c", "refs/heads/branch2").unwrap(),
        ];

        let remote_repo = "origin";

        // Crea el objeto FetchHead con las referencias simuladas
        let result = FetchHead::new(&references, remote_repo);

        // Verifica que la creación del FetchHead sea exitosa
        assert!(result.is_ok());

        let fetch_head = result.unwrap();

        // Verifica que la cantidad de entradas sea la esperada
        assert_eq!(fetch_head.entries.len(), 2);
        
        // Verifica que las entradas tengan los valores esperados
        let entry1 = &fetch_head.entries[0];
        assert_eq!(entry1.commit_hash, "93455fe53543e1dcca9533dd51d5b83656a6432c");
        assert_eq!(entry1.branch_name, "branch1");
        assert_eq!(entry1.label, Label::Merge);
        assert_eq!(entry1.remote_repo, "origin");

        let entry2 = &fetch_head.entries[1];
        assert_eq!(entry2.commit_hash, "56620fe39508e1dcca4873dd51d5b83656a9418c");
        assert_eq!(entry2.branch_name, "branch2");
        assert_eq!(entry2.label, Label::Merge);
        assert_eq!(entry2.remote_repo, "origin");

    }
    
    #[test]
    fn test_new_fetch_head_from_file() {
        let result = _read_fetch_head("./test_files/test_head");
        println!("{:?}", result);
        assert!(result.is_ok());

        let fetch_head = result.unwrap();

        // Verifica que la cantidad de entradas sea la esperada
        assert_eq!(fetch_head.entries.len(), 2);

        // Verifica que las entradas tengan los valores esperados
        let entry1 = &fetch_head.entries[0];
        assert_eq!(entry1.commit_hash, "93455fe53543e1dcca9533dd51d5b83656a6432c");
        assert_eq!(entry1.branch_name, "branch1");
        assert_eq!(entry1.label, Label::Merge);
        assert_eq!(entry1.remote_repo, "origin");

        let entry2 = &fetch_head.entries[1];
        assert_eq!(entry2.commit_hash, "56620fe39508e1dcca4873dd51d5b83656a9418c");
        assert_eq!(entry2.branch_name, "branch2");
        assert_eq!(entry2.label, Label::Merge);
        assert_eq!(entry2.remote_repo, "origin");
    }
}
