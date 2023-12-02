use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;

use crate::{consts::{CONFIG_FILE, GIT_DIR, CONFIG_REMOTE_FETCH}, git_server::GitServer};

use super::errors::CommandsError;

#[derive(Debug)]
struct BranchInfo {
    pub remote: Option<String>,
    pub merge: Option<String>,
}

impl BranchInfo {
    fn new() -> Self {
        Self {
            remote: None,
            merge: None,
        }
    }

    fn update_info(&mut self, key: &str, value: &str) -> Result<(), CommandsError>{
        match key {
            "remote" => self.remote = Some(value.to_string()),
            "merge" => self.merge = Some(value.to_string()),
            _ => return Err(CommandsError::InvalidEntryConfigFile),
        };
        Ok(())
    }

    fn format(&self) -> String
    {
        let mut format = String::new();
        match &self.remote {
            Some(value) => {
                let s = format!("\tremote = {}\n", value);
                format.extend(vec![s]);
            },
            None => (),
        }
        match &self.merge {
            Some(value) => {
                let s = format!("\tmerge = {}\n", value);
                format.extend(vec![s]);
            },
            None => (),
        }
        format
    }

    pub fn valid_attribute(attribute: &str) -> bool
    {
        matches!(attribute, "remote" | "merge")
    }

    fn get_value(&self, key: &str) -> Option<&str>
    {
        match key
        {
            "remote" => self.remote.as_deref(),
            "merge" => self.merge.as_deref(),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct RemoteInfo {
    pub url: Option<String>,
    pub fetch: Option<String>,
}

impl RemoteInfo {
    fn new() -> Self {
        Self {
            url: None,
            fetch: Some(CONFIG_REMOTE_FETCH.to_string()),
        }
    }

    fn update_info(&mut self, key: &str, value: &str, name_remote: &str) -> Result<(), CommandsError>{
        match key {
            "url" => {
                self.url = Some(value.to_string());
                let fetch  = format!("+refs/heads/*:refs/remotes/{}/*", name_remote);
                self.fetch = Some(fetch);
            },
            "fetch" => self.fetch = Some(value.to_string()),
            _ => return Err(CommandsError::InvalidEntryConfigFile),
        };
        Ok(())
    }

    fn format(&self) -> String
    {
        let mut format = String::new();
        match &self.url {
            Some(value) => {
                let s = format!("\turl = {}\n", value);
                format.extend(vec![s]);
            },
            None => (),
        }
        match &self.fetch {
            Some(value) => {
                let s = format!("\tfetch = {}\n", value);
                format.extend(vec![s]);
            },
            None => (),
        }
        format
    }

    fn get_value(&self, key: &str) -> Option<&str>
    {
        match key
        {
            "url" => self.url.as_deref(),
            "fetch" => self.fetch.as_deref(),
            _ => None,
        }
    }

    fn is_empty(&self) -> bool
    {
        self.url.is_none()
    }

    pub fn valid_attribute(attribute: &str) -> bool
    {
        matches!(attribute, "url" | "fetch")
    }
}

/// Representa la configuración de Git con secciones específicas.
///
/// La estructura almacena información de configuración en secciones, incluyendo la sección
/// "core", "remote.origin" y "branch.main".
/// La deficion de los miembros:
/// * `core`: HashMap que contiene la información de la sección "core".
/// * `remotes`: HashMap que contiene la información de la sección "remote.origin".
/// * `branch`: HashMap que contiene la información de la sección "branch.main".
///
#[derive(Debug)]
pub struct GitConfig {
    core: HashMap<String, String>,
    remotes: HashMap<String, RemoteInfo>,
    branch: HashMap<String, BranchInfo>,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl GitConfig {
    pub fn new() -> Self {
        Self {
            core: HashMap::new(),
            remotes: HashMap::new(),
            branch: HashMap::new(),
        }
    }

    /// Crea una nueva instancia de `GitConfig` basada en la configuración encontrada en el repositorio especificado.
    ///
    /// # Argumentos
    ///
    /// * `repo` - Una cadena que representa la ruta al repositorio Git.
    ///
    /// # Devolución
    ///
    /// Una instancia de `GitConfig` inicializada con la configuración leída del repositorio.
    ///
    /// # Pánicos
    ///
    /// Esta función generará un pánico si hay problemas al leer la configuración de Git desde el repositorio.
    ///
    pub fn new_from_file(repo: &str) -> Result<Self, CommandsError> {
        let path = format!("{}/{}/{}", repo, GIT_DIR, CONFIG_FILE);
        GitConfig::_new_from_file(&path)
    }

    fn _new_from_file(path: &str) -> Result<Self, CommandsError> {
        let mut git_config = GitConfig::new();
        match read_format_config(path) {
            Ok(section) => {
                for (name, attributes) in section {
                    for (key, value) in attributes {
                        git_config.add_entry(&key, &value, &name)?;
                    }
                }
                Ok(git_config)
            }
            Err(_) => Err(CommandsError::FileNotFoundConfig),
        }
    }

    /// Crea una nueva instancia de `GitConfig` a partir de la configuración del servidor remoto.
    ///
    /// # Parámetros
    ///
    /// - `server`: La configuración del servidor remoto representada por un objeto `GitServer`.
    ///
    /// # Errores
    ///
    /// Devuelve un error [`CommandsError`] si hay algún problema al configurar el `GitConfig`.
    ///
    pub fn new_from_server(server: &GitServer) -> Result<Self, CommandsError>
    {
        let mut git_config = GitConfig::new();
        git_config.add_entry("url", &server.src_repo.to_string(), "remote origin")?;
        for refs in server.get_references()
        {
            let name = refs.get_name().to_string();
            let mut branch_info = BranchInfo::new();
            branch_info.update_info("remote", "origin")?;
            branch_info.update_info("merge", &name)?;
            git_config.branch.insert(name, branch_info);
        }
        Ok(git_config)
    }

    /// Agrega una entrada a una sección específica de la configuración Git.
    ///
    /// # Argumentos
    ///
    /// * `key`: La clave de la entrada que se va a agregar.
    /// * `value`: El valor de la entrada que se va a agregar.
    /// * `section`: La sección a la que pertenece la entrada.
    ///
    pub fn add_entry(&mut self, key: &str, value: &str, section: &str) -> Result<(), CommandsError> {
        if section == "core" {
            self.core.insert(key.to_string(), value.to_string());
            return Ok(());
        };
        let parts: Vec<&str> = section.split_whitespace().collect();
        if parts.len() != 2 {
            println!("parts: {:?}", parts);
            return Err(CommandsError::InvalidEntryConfigFile);
        }
        match parts[0].trim() {
            "remote" => {
                let name = get_name_seccion(section)
                .ok_or(CommandsError::InvalidEntryConfigFile)?;

                if !RemoteInfo::valid_attribute(key)
                {
                    return Err(CommandsError::InvalidEntryConfigFile);
                }
                if !self.remotes.contains_key(&name) {
                    self.remotes.insert(name.to_string(), RemoteInfo::new());
                }
                let remote_info = match self.remotes.get_mut(&name)
                {
                    Some(remote_info) => remote_info,
                    None => return Err(CommandsError::InvalidEntryConfigFile),
                };
                remote_info.update_info(key, value, &name)?;
                // self.remotes.update_info(key, value)?;
                Ok(())
            }
            "branch" => {
                let name = get_name_seccion(section)
                    .ok_or(CommandsError::InvalidEntryConfigFile)?;

                if !BranchInfo::valid_attribute(key)
                {
                    return Err(CommandsError::InvalidEntryConfigFile);
                }
                if !self.branch.contains_key(&name) {
                    self.branch.insert(name.to_string(), BranchInfo::new());
                }
                let branch_info = match self.branch.get_mut(&name)
                {
                    Some(branch_info) => branch_info,
                    None => return Err(CommandsError::InvalidEntryConfigFile),
                };
                branch_info.update_info(key, value)?;
                Ok(())
            }
            _ => Err(CommandsError::InvalidEntryConfigFile),
        }
    }

    /// Escribe la configuración Git en un archivo especificado.
    ///
    /// # Argumentos
    ///
    /// * `file_path`: La ruta del archivo en el que se escribirá la configuración.
    ///
    /// # Errores
    ///
    /// Devuelve un resultado `io::Result` indicando si la operación fue exitosa o si se produjo un error
    /// al escribir en el archivo.
    ///
    pub fn write_to_file(&self, file_path: &str) -> Result<(), CommandsError> {
        match self._write_to_file(file_path) {
            Ok(_) => Ok(()),
            Err(_) => Err(CommandsError::CreateGitConfig),
        }
    }

    fn _write_to_file(&self, file_path: &str) -> io::Result<()> {
        let mut file = File::create(file_path)?;

        // Write core section
        if !self.core.is_empty()
        {
            writeln!(file, "[core]")?;
            for (key, value) in &self.core {
                writeln!(file, "\t{} = {}", key, value)?;
            }
        };

        // Write remote "origin" section
        if !self.remotes.is_empty()
        {
            for (name, value) in &self.remotes {
                if value.is_empty()
                {
                    continue;
                }
                writeln!(file, "[remote \"{}\"]", name)?;
                write!(file, "{}", value.format())?;
            }
        }

        // Write branch "main" section
        if !self.branch.is_empty()
        {        
            for (name, value) in &self.branch {
                writeln!(file, "[branch \"{}\"]", name)?;
                write!(file, "{}", value.format())?;
            }
        }

        Ok(())
    }

    /// Obtiene la URL del repositorio remoto con el nombre especificado.
    ///
    /// Esta función busca en la configuración de remotos almacenada en el objeto `GitConfig` y devuelve
    /// la URL del repositorio remoto correspondiente al nombre proporcionado.
    ///
    /// # Arguments
    ///
    /// * `name` - Nombre del repositorio remoto cuya URL se desea obtener.
    ///
    /// # Returns
    ///
    /// Retorna un resultado que contiene la URL del repositorio remoto si se encuentra, o un error
    /// si no se encuentra la configuración del remoto o si la URL está ausente en la configuración.
    ///
    pub fn get_remote_url_by_name(&self, name_remote: &str) -> Result<String, CommandsError> {
        let remote = match self.remotes.get(name_remote)
        {
            Some(remote) => remote,
            None => return Err(CommandsError::MissingUrlConfig),
        };
        match &remote.url {
            Some(url) => Ok(url.to_string()),
            None => Err(CommandsError::MissingUrlConfig),
        }
    }

    /// Obtiene el nombre del repositorio remoto asociado a una rama específica.
    ///
    /// Esta función busca la configuración de la rama almacenada en el objeto `GitConfig` y devuelve
    /// el nombre del repositorio remoto asociado a la rama proporcionada.
    ///
    /// # Arguments
    ///
    /// * `name_branch` - Nombre de la rama cuyo repositorio remoto se desea obtener.
    ///
    /// # Returns
    ///
    /// Retorna un resultado que contiene el nombre del repositorio remoto si se encuentra, o un error
    /// si no se encuentra la configuración de la rama o si el remoto está ausente en la configuración.
    ///
    pub fn get_remote_by_branch_name(&self, name_branch: &str) -> Result<String, CommandsError>
    {
        let branch = match self.branch.get(name_branch)
        {
            Some(b) => b,
            None => return Err(CommandsError::NoTrackingInformationForBranch),
        };

        match &branch.remote
        {
            Some(remote) => Ok(remote.to_string()),
            None => Err(CommandsError::NoTrackingInformationForBranch),
        }
    }

    pub fn get_name_remote_by_url(&self, url: &str) -> Option<String>
    {
        for (name, remote_info) in &self.remotes
        {
            if let Some(remote_url) = remote_info.get_value("url")
            {
                if remote_url == url
                {
                    return Some(name.to_string());
                }
            }
        }
        None
    }

    /// Obtiene la URL del repositorio remoto asociado a una rama específica.
    ///
    /// Esta función utiliza `get_remote_by_branch_name` para obtener el nombre del repositorio remoto
    /// asociado a la rama y luego utiliza `get_remote_url_by_name` para obtener la URL correspondiente.
    ///
    /// # Arguments
    ///
    /// * `name_branch` - Nombre de la rama cuyo repositorio remoto se desea obtener.
    ///
    /// # Returns
    ///
    /// Retorna un resultado que contiene la URL del repositorio remoto si se encuentra, o un error
    /// si no se encuentra la configuración de la rama o si la URL está ausente en la configuración del remoto.
    ///
    /// # Errors
    ///
    /// Retorna un error del tipo `CommandsError` en caso de que no se encuentre la configuración de la rama o si
    /// la URL está ausente en la configuración del remoto.
    ///
    pub fn get_branch_url_by_name(&self, name_branch: &str) -> Result<String, CommandsError>
    {
        let remote = self.get_remote_by_branch_name(name_branch)?;

        self.get_remote_url_by_name(&remote)
    }

    /// Obtiene el valor asociado a una clave en una sección específica de la configuración Git.
    ///
    /// # Argumentos
    ///
    /// * `section`: La sección a la que pertenece la entrada.
    /// * `key`: La clave de la entrada cuyo valor se va a obtener.
    ///
    /// # Retorno
    ///
    /// Devuelve `Some(&String)` si la entrada existe, o `None` si no se encuentra.
    ///
    pub fn get_value(&self, section: &str, key: &str) -> Option<&str>
    {
        if section == "core"
        {
            return self.core.get(key).map(|x| x.as_str())
        }
        let parts: Vec<&str> = section.split_whitespace().collect();
        if parts.len() != 2 {
            println!("parts: {:?}", parts);
            return None;
        }
        match parts[0].trim() {
            "remote" => {
                let name = get_name_seccion(section)?;
                match self.remotes.get(&name)
                {
                    Some(r) => r.get_value(key),
                    None => None,
                }
            },
            "branch" => {
                let name = get_name_seccion(section)?;
                match self.branch.get(&name)
                {
                    Some(b) => b.get_value(key),
                    None => None,
                }
            }
            _ => None,
        }
    }

    /// Agrega o actualiza la información de un repositorio remoto en la configuración Git.
    ///
    /// Esta función agrega un nuevo repositorio remoto o actualiza la información de uno existente
    /// en la configuración almacenada en el objeto `GitConfig`. El nombre y la URL del remoto se
    /// proporcionan como argumentos. Si ya existe un remoto con el mismo nombre, se reemplazará con la
    /// nueva información.
    ///
    /// # Arguments
    ///
    /// * `name_remote` - Nombre del repositorio remoto.
    /// * `url` - URL del repositorio remoto.
    ///
    /// # Returns
    ///
    /// Retorna un resultado indicando si la operación fue exitosa o si ocurrió un error al actualizar la
    /// configuración del remoto.
    ///
    pub fn add_remote(&mut self, name_remote: &str, url: &str) -> Result<(), CommandsError>
    {
        if self.remotes.contains_key(name_remote)
        {
            self.remotes.remove(name_remote);
        }
        let mut remote_info = RemoteInfo::new();
        remote_info.update_info("url", url, name_remote)?;
        self.remotes.insert(name_remote.to_string(), remote_info);
        Ok(())
    }

    /// Agrega o actualiza la información de una rama en la configuración Git.
    ///
    /// Esta función agrega una nueva rama o actualiza la información de una existente
    /// en la configuración almacenada en el objeto `GitConfig`. El nombre de la rama, el remoto
    /// asociado y la rama de merge se proporcionan como argumentos. Si ya existe una rama con
    /// el mismo nombre, se reemplazará con la nueva información.
    ///
    /// # Arguments
    ///
    /// * `name_branch` - Nombre de la rama.
    /// * `remote` - Nombre del repositorio remoto asociado a la rama.
    /// * `merge` - Nombre de la rama de merge asociada.
    ///
    /// # Returns
    ///
    /// Retorna un resultado indicando si la operación fue exitosa o si ocurrió un error al actualizar la
    /// configuración de la rama.
    ///
    /// # Errors
    ///
    /// Retorna un error del tipo `CommandsError` si ocurre algún problema al agregar o actualizar la información de la rama.
    ///
    pub fn add_branch(&mut self, name_branch: &str, remote: &str, merge: &str) -> Result<(), CommandsError>
    {
        if self.branch.contains_key(name_branch)
        {
            self.branch.remove(name_branch);
        }
        let mut branch_info = BranchInfo::new();
        branch_info.update_info("remote", remote)?;
        branch_info.update_info("merge", merge)?;
        self.branch.insert(name_branch.to_string(), branch_info);
        Ok(())
    }

    /// Elimina una rama del repositorio local.
    ///
    /// # Arguments
    ///
    /// * `name_branch` - Nombre de la rama a eliminar.
    ///
    /// # Returns
    ///
    /// Retorna `Ok(())` si la rama se elimina correctamente, o un error `CommandsError` si la rama no se encuentra.
    ///
    /// # Errors
    ///
    /// Puede retornar un error `CommandsError::BranchNotFound` si la rama no existe en la configuración del repositorio.
    ///
    pub fn delete_branch(&mut self, name_branch: &str) -> Result<(), CommandsError>
    {
        if !self.branch.contains_key(name_branch)
        {
            return Err(CommandsError::BranchNotFound);
        }
        self.branch.remove(name_branch);
        Ok(())
    }

    /// Elimina un remoto del repositorio local.
    ///
    /// # Arguments
    ///
    /// * `name_remote` - Nombre del remoto a eliminar.
    ///
    /// # Returns
    ///
    /// Retorna `Ok(())` si el remoto se elimina correctamente, o un error `CommandsError` si el remoto no se encuentra.
    ///
    /// # Errors
    ///
    /// Puede retornar un error `CommandsError::RemoteNotFound` si el remoto no existe en la configuración del repositorio.
    ///
    pub fn delete_remote(&mut self, name_remote: &str) -> Result<(), CommandsError>
    {
        if !self.remotes.contains_key(name_remote)
        {
            return Err(CommandsError::RemoteNotFound);
        }
        self.remotes.remove(name_remote);

        // Eliminar las ramas que apuntan al remoto
        let mut branches_to_delete = Vec::new();
        for (name_branch, branch_info) in &self.branch
        {
            if branch_info.remote.as_deref() == Some(name_remote)
            {
                branches_to_delete.push(name_branch.to_string());
            }
        }
        for name_branch in branches_to_delete
        {
            self.branch.remove(&name_branch);
        }

        Ok(())
    }

    pub fn get_remotes_in_use(&self) -> HashSet<String>
    {
        let mut remotes = HashSet::new();
        for (_, branch_info) in &self.branch
        {
            if let Some(remote) = branch_info.get_value("remote")
            {
                remotes.insert(remote.to_string());
            }
        }
        remotes
    }

    pub fn get_remote_branch_ref(&self, name_branch: &str) -> Option<String>
    {
        let branch_info = match self.branch.get(name_branch)
        {
            Some(branch_info) => branch_info,
            None => return None,
        };
        let remote = match branch_info.get_value("remote")
        {
            Some(remote) => remote,
            None => return None,
        };
        let remote_info = match self.remotes.get(remote)
        {
            Some(remote_info) => remote_info,
            None => return None,
        };
        let fetch = match remote_info.get_value("fetch")
        {
            Some(fetch) => fetch,
            None => return None,
        };
        let parts = fetch.split(":").collect::<Vec<&str>>();
        if parts.len() != 2
        {
            return None;
        }
        let location: &str = parts[1].trim();
        let location = &location.replace("*", name_branch);
        Some(location.to_string())
    }
}

// Lee un archivo de configuración en formato especificado y devuelve un HashMap.
///
/// El archivo de configuración debe tener secciones entre corchetes y atributos en formato clave=valor.
/// Las secciones actúan como claves en el HashMap externo, y los atributos como claves en los HashMap internos.
///
/// # Ejemplo
///
/// ```ignore
/// [core]
///     repositoryformatversion = 0
///     filemode = true
///     bare = false
///     logallrefupdates = true
/// [remote "origin"]
///     url = git@github.com:Brubrux/Proyecto-AnInfo-.git
///     fetch = +refs/heads/*:refs/remotes/origin/*
/// [branch "main"]
///     remote = origin
///     merge = refs/heads/main
/// [branch "6-feature-menu-principal-interfaz-del-juego"]
///     remote = origin
///     merge = refs/heads/6-feature-menu-principal-interfaz-del-juego
/// ```
///
/// # Arguments
///
/// * `path` - Ruta al archivo de configuración.
///
/// # Returns
///
/// Retorna un Resultado que contiene un HashMap donde las claves son las secciones y los valores son HashMaps
/// de atributos en formato clave=valor. En caso de error, se devuelve un error CommandsError.
/// 
fn read_format_config(path: &str) -> Result<HashMap<String, HashMap<String, String>>, CommandsError> {
    let content = match fs::read_to_string(path)
    {
        Ok(content) => content,
        Err(_) => return Err(CommandsError::FileNotFoundConfig),
    };
    let mut result = HashMap::new();
    let mut current_section = String::new();
    let mut current_attributes = HashMap::new();

    for line in content.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#'){
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            if !current_section.is_empty() {
                result.insert(current_section, current_attributes);
            }
            current_attributes = HashMap::new();
            current_section = line[1..line.len() - 1].to_string();
            continue;
        } 
        let parts: Vec<&str> = line.splitn(2, '=').collect();

        if parts.len() != 2 {
            return Err(CommandsError::InvalidConfigFile);
        }
        let key = parts[0].trim();
        let value = parts[1].trim();
        current_attributes.insert(key.to_string(), value.to_string());
    }
    if !current_section.is_empty() {
        result.insert(current_section, current_attributes);
    };
    Ok(result)
}


/// Extrae y devuelve el nombre de una sección a partir de una cadena dada.
///
/// La función toma una cadena que representa una sección y retorna el nombre de la sección si el formato
/// es válido. Las secciones válidas son "remote" y "branch". El nombre de la sección debe estar rodeado
/// por comillas dobles ("") o comillas simples ('').
///
/// # Arguments
///
/// * `section` - Cadena que representa una sección con el formato "tipo nombre".
///
/// # Returns
///
/// Retorna `Some` con el nombre de la sección si es válido, o `None` si el formato es incorrecto o no se encuentra
/// un nombre válido.
///
fn get_name_seccion(section: &str) -> Option<String>
{
    let parts: Vec<&str> = section.split_whitespace().collect();
    if parts.len() != 2 {
        println!("parts: {:?}", parts);
        return None;
    }
    match parts[0].trim() {
        "remote" | "branch" => {
            let name = parts[1];
            let name = name.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace()).to_string();
            Some(name.to_string())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;

    #[test]
    fn add_entry_valid_core() {
        let mut git_config = GitConfig::new();
        git_config.add_entry("repositoryformatversion", "0", "core").unwrap();
        assert_eq!(
            git_config.core.get("repositoryformatversion"),
            Some(&"0".to_string())
        );
    }

    #[test]
    fn add_entry_valid_remotes() {
        let mut git_config = GitConfig::new();
        git_config.add_entry("url", "git@github.com:example/repo.git", "remote origin").unwrap();
        assert_eq!(
            git_config.get_remote_url_by_name("origin").unwrap(),
            "git@github.com:example/repo.git".to_string()
        );
    }

    #[test]
    fn add_entry_valid_branch() {
        let mut git_config = GitConfig::new();
        git_config.add_entry("remote", "origin", "branch \"main\"").unwrap();
        assert_eq!(
            git_config.get_remote_by_branch_name("main").unwrap(),
            "origin"
        );
        assert!(true)
    }

    #[test]
    fn add_entry_invalid_key() {
        let mut git_config = GitConfig::new();
        let _ = git_config.add_entry("invalid", "origin", "branch \"main\"");
        assert!(git_config.core.is_empty());
        assert!(git_config.remotes.is_empty());
        assert!(git_config.branch.is_empty());
    }

    #[test]
    fn add_entry_invalid_section() {
        let mut git_config = GitConfig::new();
        let _ = git_config.add_entry("bare", "false", "invalid");
        assert!(git_config.core.is_empty());
        assert!(git_config.remotes.is_empty());
        assert!(git_config.branch.is_empty());
    }

    #[test]
    fn add_entry_error() {
        let mut git_config = GitConfig::new();
        assert!(git_config.add_entry("bare", "false", "invalid").is_err());
    }

    #[test]
    fn test_new_from_file_nonexistent_file() {
        let result = GitConfig::new_from_file("nonexistent_file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_value_core() {
        let mut git_config = GitConfig::new();
        let _ = git_config.add_entry("repositoryformatversion", "0", "core");
        assert_eq!(
            git_config.get_value("core", "repositoryformatversion").unwrap(),
            "0"
        );
    }

    #[test]
    fn test_get_value_remotes() {
        let mut git_config = GitConfig::new();
        let _ = git_config.add_entry("url", "git@github.com:example/repo.git", "remote origin");
        assert_eq!(
            git_config.get_value("remote origin", "url").unwrap(),
            "git@github.com:example/repo.git"
        );
    }

    #[test]
    fn test_get_value_branch_remote() {
        let mut git_config = GitConfig::new();
        let _ = git_config.add_entry("remote", "origin", "branch \"main\"");
        println!("{:?}", git_config);
        assert_eq!(
            git_config.get_value("branch \"main\"", "remote").unwrap(),
            "origin"
        );
    }

    #[test]
    fn test_get_value_unknown_section() {
        let git_config = GitConfig::new();
        assert_eq!(git_config.get_value("unknown_section", "key"), None);
    }

    #[test]
    fn test_write_to_file() {
        let mut git_config = GitConfig::new();
        let _ = git_config.add_entry("url", "git@github.com:example/repo.git", "remote origin");
        let _ = git_config.add_entry("remote", "origin", "branch \"main\"");

        let file_path = "./test_files/test_config";
        let _ = git_config.write_to_file(file_path);

        let mut file_content = String::new();
        let mut file = File::open(file_path).expect("Could not open file");
        file.read_to_string(&mut file_content)
            .expect("Could not read file");

        let expected_content = "\
                                [remote \"origin\"]\n\
                                \turl = git@github.com:example/repo.git\n\
                                [branch \"main\"]\n\
                                \tremote = origin\n";
                                
        // Cleanup
        fs::remove_file(file_path).expect("No se pudo eliminar el config del tests");
        assert_eq!(file_content, expected_content);
    }

    #[test]
    fn test_read_git_config_from_file() {
        let file_path = "./test_files/config";
        let config = GitConfig::_new_from_file(file_path).unwrap();

        assert_eq!(config.branch.len(), 5);
        assert!(!config.remotes.is_empty());
        assert!(!config.remotes.is_empty());
    }

    #[test]
    fn test_get_name_seccion_valid_remote() {
        assert_eq!(get_name_seccion("remote \"origin\""), Some("origin".to_string()));
    }

    #[test]
    fn test_get_name_seccion_valid_branch() {
        assert_eq!(get_name_seccion("branch 'main'"), Some("main".to_string()));
    }

    #[test]
    fn test_get_name_seccion_valid_single_quote() {
        assert_eq!(get_name_seccion("branch 'feature'"), Some("feature".to_string()));
    }

    #[test]
    fn add_branch_success() {
        let mut config = GitConfig::new();

        // Agregar una rama nueva
        let result = config.add_branch("main", "origin", "refs/heads/main");
        assert!(result.is_ok());

        // Verificar que la rama se ha agregado correctamente
        assert_eq!(config.branch.len(), 1);
        assert!(config.branch.contains_key("main"));

        let branch_info = config.branch.get("main").unwrap();
        assert_eq!(branch_info.remote, Some("origin".to_string()));
        assert_eq!(branch_info.merge, Some("refs/heads/main".to_string()));
    }

    #[test]
    fn add_branch_update_existing() {
        let mut config = GitConfig::new();

        // Agregar una rama existente
        config.add_branch("main", "origin", "refs/heads/main").unwrap();

        // Actualizar la información de la rama
        let result = config.add_branch("main", "upstream", "refs/heads/upstream");
        assert!(result.is_ok());

        // Verificar que la rama se ha actualizado correctamente
        assert_eq!(config.branch.len(), 1);
        assert!(config.branch.contains_key("main"));

        let branch_info = config.branch.get("main").unwrap();
        assert_eq!(branch_info.remote, Some("upstream".to_string()));
        assert_eq!(branch_info.merge, Some("refs/heads/upstream".to_string()));
    }

    #[test]
    fn add_branch_duplicate() {
        let mut config = GitConfig::new();

        // Agregar una rama nueva
        config.add_branch("main", "origin", "refs/heads/main").unwrap();

        let result = config.add_branch("main", "upstream", "refs/heads/upstream");
        assert!(result.is_ok());

        // Verificar que la rama se ha reemplazado
        assert_eq!(config.branch.len(), 1);
        assert!(config.branch.contains_key("main"));

        let branch_info = config.branch.get("main").unwrap();
        assert_eq!(branch_info.remote, Some("upstream".to_string()));
        assert_eq!(branch_info.merge, Some("refs/heads/upstream".to_string()));
    }

    #[test]
    fn test_delete_branch() {
        let mut git_config = GitConfig::new();
        git_config.add_remote("origin", "github").unwrap();
        git_config.add_branch("main", "origin", "refs/heads/main").unwrap();

        assert!(git_config.get_remote_by_branch_name("main").is_ok());

        assert_eq!(git_config.delete_branch("main"), Ok(()));
        assert_eq!(git_config.get_remote_by_branch_name("main"), Err(CommandsError::NoTrackingInformationForBranch));
    }

    #[test]
    fn test_delete_remote() {
        let mut git_config = GitConfig::new();
        git_config.add_remote("origin", "github").unwrap();
        git_config.add_branch("main", "origin", "refs/heads/main").unwrap();

        assert!(git_config.get_remote_by_branch_name("main").is_ok());
        assert!(git_config.get_remote_url_by_name("origin").is_ok());

        assert_eq!(git_config.delete_remote("origin"), Ok(()));
        assert!(git_config.get_remote_url_by_name("main").is_err());
        assert_eq!(git_config.get_remote_by_branch_name("main"), Err(CommandsError::NoTrackingInformationForBranch));
    }

    #[test]
    fn test_write_and_to_file() {
        let mut git_config = GitConfig::new();
        git_config.add_remote("origin", "Repository_1").unwrap();
        git_config.add_branch("main", "origin", "refs/heads/main").unwrap();
        git_config.add_remote("upstream", "Repository_2").unwrap();
        git_config.add_branch("feature", "upstream", "refs/heads/feature").unwrap();
        git_config.add_branch("master", "upstream", "refs/heads/master").unwrap();
        git_config.delete_branch("master").unwrap();
        let file_path = "./test_files/test_config_2";
        let _ = git_config.write_to_file(file_path);

        let git_config = GitConfig::_new_from_file(file_path).unwrap();
        fs::remove_file(file_path).expect("No se pudo eliminar el config del tests");
        // Cleanup
        assert_eq!(git_config.branch.len(), 2);
        assert_eq!(git_config.remotes.len(), 2);
        assert_eq!(git_config.get_remote_url_by_name("origin").unwrap(), "Repository_1");
        assert_eq!(git_config.get_remote_url_by_name("upstream").unwrap(), "Repository_2");
        assert_eq!(git_config.get_remote_by_branch_name("main").unwrap(), "origin");
        assert_eq!(git_config.get_remote_by_branch_name("feature").unwrap(), "upstream");

    }

    #[test]
    fn test_get_remote_branch_ref()
    {
        let mut git_config = GitConfig::new();
        git_config.add_remote("origin", "Repository_1").unwrap();
        git_config.add_branch("main", "origin", "refs/heads/main").unwrap();
        git_config.add_remote("upstream", "Repository_2").unwrap();
        git_config.add_branch("feature", "upstream", "refs/heads/feature").unwrap();
        git_config.add_branch("master", "upstream", "refs/heads/master").unwrap();
        git_config.delete_branch("master").unwrap();

        assert_eq!(git_config.get_remote_branch_ref("main"), Some("refs/remotes/origin/main".to_string()));
        assert_eq!(git_config.get_remote_branch_ref("feature"), Some("refs/remotes/upstream/feature".to_string()));
        assert_eq!(git_config.get_remote_branch_ref("master"), None);
    }
}
