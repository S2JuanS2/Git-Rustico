use std::{io::{self, Write, BufRead}, fs};

use super::errors::CommandsError;


/// Enum que representa las posibles etiquetas para FetchHeadEntry.
pub enum Label {
    NotForMerge,
    Merge,
}

impl ToString for Label {
    fn to_string(&self) -> String {
        match self {
            Label::NotForMerge => "not-for-merge".to_string(),
            Label::Merge => "".to_string(),
        }
    }
}

/// Struct que representa una entrada en el archivo FETCH_HEAD.
pub struct FetchHeadEntry {
    commit_hash: String,
    branch_name: String,
    label: Label,
    remote_repo: String,
}

impl FetchHeadEntry {
    /// Crea una nueva FetchHeadEntry.
    ///
    /// # Argumentos
    ///
    /// * `commit_hash` - El hash del commit asociado con la entrada.
    /// * `branch_name` - El nombre de la rama asociada con la entrada.
    /// * `label` - Una cadena que representa la etiqueta de la entrada ("not-for-merge" o "").
    ///
    /// # Retorno
    ///
    /// Una nueva FetchHeadEntry.
    /// 
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

pub struct FetchHead {
    entries: Vec<FetchHeadEntry>,
}

impl FetchHead {

    /// Crea una nueva instancia de FetchHead a partir de las referencias dadas.
    ///
    /// # Argumentos
    ///
    /// * `references` - Vec de tuplas que contienen el hash del commit y el nombre de la rama.
    /// * `repo_local` - Ruta local del repositorio.
    /// * `remote_repo` - Ruta remota del repositorio remoto.
    ///
    /// # Retorno
    ///
    /// Una instancia de FetchHead creada a partir de las referencias proporcionadas.
    ///
    /// # Errores
    ///
    /// Devuelve un error de tipo CommandsError si ocurre algún problema durante la creación.
    /// 
    pub fn new(
        references: Vec<(String, String)>,
        remote_repo: &str
    ) -> Result<FetchHead, CommandsError> {
        let mut entries = Vec::new();
        for (commit_hash, branch_name) in references {
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
            let line = format!(
                "{}\t{}\tbranch '{}' of github.com:{}\n",
                entry.commit_hash,
                entry.label.to_string(),
                entry.branch_name,
                entry.remote_repo
            );
            file.write_all(line.as_bytes())?;
        }
        Ok(())
    }

    pub fn new_from_file(repo_path: &str) -> Result<FetchHead, CommandsError> {
    let repo = format!("{}/.git", repo_path);
    _read_fetch_head(&repo)
}
}


// fn 

// /// Lee el contenido del archivo FETCH_HEAD y devuelve un vector con las referencias.
// ///
// /// # Argumentos
// ///
// /// * `repo_path` - Ruta del repositorio.
// /// 
// /// # Retorno
// /// 
// /// Devuelve un vector con las referencias del repositorio.
// /// Vec<(String_1, String_2, String_3)>
// /// * String_1: Hash del commit
// /// * String_2: Modo de merge
// /// * String_3: Nombre de la rama en github
// ///
// /// # Errores
// ///
// /// Devuelve un error de tipo `CommandsError` si no puede leer o interpretar el contenido del archivo.
// ///
// // pub fn read_fetch_head(repo_path: &str) -> Result<Vec<FetchHead>, CommandsError> {
// //     let repo = format!("{}/.git", repo_path);
// //     match _read_fetch_head(&repo)
// //     {
// //         Ok(result) => Ok(result),
// //         Err(_) => Err(CommandsError::ReadFetchHEAD),
// //     }
// // }

// /// Función auxiliar que implementa la lógica real para leer FETCH_HEAD.
// ///
// /// # Argumentos
// ///
// /// * `path` - Ruta donde se encuentra el archivo FETCH_HEAD.
// ///
// /// # Errores
// ///
// /// Devuelve un error de tipo `io::Error` si no puede abrir o leer el archivo FETCH_HEAD.
// ///
pub fn _read_fetch_head(path: &str) -> Result<FetchHead, CommandsError> {
    let fetch_head_path = format!("{}/FETCH_HEAD", path);
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
