use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;

use crate::consts::{CONFIG_FILE, GIT_DIR};

use super::errors::CommandsError;

#[derive(Debug)]
struct BranchInfo {
    remote: Option<String>,
    merge: Option<String>,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self::new()
    }
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
}


/// Representa la configuración de Git con secciones específicas.
///
/// La estructura almacena información de configuración en secciones, incluyendo la sección
/// "core", "remote.origin" y "branch.main".
/// La deficion de los miembros:
/// * `core`: HashMap que contiene la información de la sección "core".
/// * `remote_origin`: HashMap que contiene la información de la sección "remote.origin".
/// * `branch`: HashMap que contiene la información de la sección "branch.main".
///
#[derive(Debug)]
pub struct GitConfig {
    core: HashMap<String, String>,
    remote_origin: HashMap<String, String>,
    branch: HashMap<String, BranchInfo>,
}

impl GitConfig {
    pub fn new() -> Self {
        Self {
            core: HashMap::new(),
            remote_origin: HashMap::new(),
            branch: HashMap::new(),
        }
    }

    // pub fn new_from_lines(lines: Vec<String>) -> Self {
    //     let mut git_config = GitConfig::new();
    //     for line in lines {
    //         git_config.parse_line(&line);
    //     }
    //     git_config
    // }

    pub fn new_from_file(repo: &str) -> Result<Self, CommandsError> {
        let path = format!("{}/{}/{}", repo, GIT_DIR, CONFIG_FILE);
        GitConfig::_new_from_file(&path)
    }

    fn _new_from_file(path: &str) -> Result<Self, CommandsError> {
        let mut git_config = GitConfig::new();
        match read_format_config(&path) {
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
    // pub fn new_from_file(repo: &str) -> Result<Self, CommandsError> {
    //     let path = format!("{}/{}/{}", repo, GIT_DIR, CONFIG_FILE);
    //     match GitConfig::read_git_config(&path)  {
    //         Ok(config) => Ok(config),
    //         Err(_) => Err(CommandsError::FileNotFoundConfig),
    //     }
    // }

    /// Analiza una línea de configuración Git y actualiza las secciones correspondientes.
    ///
    /// # Argumentos
    ///
    /// * `line`: Una cadena que representa una línea de configuración Git en el formato "clave=valor".
    ///
    // pub fn parse_line(&mut self, line: &str) {
    //     let parts: Vec<&str> = line.splitn(2, '=').collect();
    //     if parts.len() == 2 {
    //         let key = parts[0].trim();
    //         let value = parts[1].trim();

    //         match key {
    //             "repositoryformatversion" | "filemode" | "bare" | "logallrefupdates" => {
    //                 self.core.insert(key.to_string(), value.to_string());
    //             }
    //             "url" | "fetch" => {
    //                 self.remote_origin
    //                     .insert(key.to_string(), value.to_string());
    //             }
    //             "remote" | "merge" => {
    //                 self.branch.insert(key.to_string(), value.to_string());
    //             }
    //             _ => {}
    //         }
    //     }
    // }

    /// Crea un GitConfig desde un archivo.
    ///
    /// # Argumentos
    ///
    /// * `file_path`: La ruta del archivo que contiene la configuración Git.
    ///
    /// # Errores
    ///
    /// Devuelve un resultado `Result` indicando si la operación fue exitosa o si se produjo un error
    /// al leer el archivo.
    ///
    // fn read_git_config(file_path: &str) -> Result<Self, std::io::Error> {
    //     let mut git_config = GitConfig::new();
    //     if Path::new(file_path).exists() {
    //         let content = fs::read_to_string(file_path)?;

    //         for line in content.lines() {
    //             git_config.parse_line(line);
    //         }
    //     }

    //     Ok(git_config)
    // }

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
                self.remote_origin.insert(key.to_string(), value.to_string());
                Ok(())
            }
            "branch" => {
                let name = parts[1].trim();
                if !name.starts_with("\"") || !name.ends_with("\"")
                {
                    return Err(CommandsError::InvalidEntryConfigFile);
                }
                let name = name[1..name.len() - 1].to_string();
                if !self.branch.contains_key(&name) {
                    self.branch.insert(name.clone(), BranchInfo::new());
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
    /// Obtiene el valor asociado a una clave en una sección específica de la configuración Git.
    ///
    /// # Argumentos
    ///
    /// * `key`: La clave de la entrada cuyo valor se va a obtener.
    /// * `section`: La sección a la que pertenece la entrada.
    ///
    /// # Retorno
    ///
    /// Devuelve `Some(&String)` si la entrada existe, o `None` si no se encuentra.
    ///
    // pub fn get_value(&self, key: &str, section: &str) -> Option<&String> {
    //     match section {
    //         "core" => self.core.get(key),
    //         "remote.origin" => self.remote_origin.get(key),
    //         "branch.main" => self.branch.get(key),
    //         _ => None,
    //     }
    // }

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
    // pub fn write_to_file(&self, file_path: &str) -> Result<(), CommandsError> {
    //     match self._write_to_file(file_path) {
    //         Ok(_) => Ok(()),
    //         Err(_) => Err(CommandsError::CreateGitConfig),
    //     }
    // }

    // fn _write_to_file(&self, file_path: &str) -> io::Result<()> {
    //     let mut file = File::create(file_path)?;

    //     // Write core section
    //     for (key, value) in &self.core {
    //         writeln!(file, "{} = {}", key, value)?;
    //     }

    //     // Write remote "origin" section
    //     for (key, value) in &self.remote_origin {
    //         writeln!(file, "[remote \"origin\"]")?;
    //         writeln!(file, "    {} = {}", key, value)?;
    //     }

    //     // Write branch "main" section
    //     for (key, value) in &self.branch {
    //         writeln!(file, "[branch \"main\"]")?;
    //         writeln!(file, "    {} = {}", key, value)?;
    //     }

    //     Ok(())
    // }

    pub fn get_remote_repo(&self) -> Result<&String, CommandsError> {
        match self.remote_origin.get("url") {
            Some(url) => Ok(url),
            None => Err(CommandsError::MissingUrlConfig),
        }
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

        if line.starts_with("[") && line.ends_with("]") {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    // #[test]
    // fn parse_line_valid_core_key_value() {
    //     let mut git_config = GitConfig::new();
    //     git_config.parse_line("repositoryformatversion = 0");
    //     assert_eq!(
    //         git_config.core.get("repositoryformatversion"),
    //         Some(&"0".to_string())
    //     );
    // }

    // #[test]
    // fn parse_line_valid_remote_origin_key_value() {
    //     let mut git_config = GitConfig::new();
    //     git_config.parse_line("url = git@github.com:example/repo.git");
    //     assert_eq!(
    //         git_config.remote_origin.get("url"),
    //         Some(&"git@github.com:example/repo.git".to_string())
    //     );
    // }

    // #[test]
    // fn parse_line_valid_branch_key_value() {
    //     let mut git_config = GitConfig::new();
    //     git_config.parse_line("remote = origin");
    //     assert_eq!(
    //         git_config.branch.get("remote"),
    //         Some(&"origin".to_string())
    //     );
    // }

    // #[test]
    // fn parse_line_invalid_key_value() {
    //     let mut git_config = GitConfig::new();
    //     git_config.parse_line("invalid_key = invalid_value");
    //     assert!(git_config.core.is_empty());
    //     assert!(git_config.remote_origin.is_empty());
    //     assert!(git_config.branch.is_empty());
    // }

    // #[test]
    // fn parse_line_invalid_syntax() {
    //     let mut git_config = GitConfig::new();
    //     git_config.parse_line("invalid_syntax_without_equal_sign");
    //     assert!(git_config.core.is_empty());
    //     assert!(git_config.remote_origin.is_empty());
    //     assert!(git_config.branch.is_empty());
    // }

    // #[test]
    // fn test_read_git_config() {
    //     // Crea un archivo de prueba temporal con contenido de configuración Git
    //     let temp_file_path = "./test_files/config_2";
    //     let mut temp_file =
    //         std::fs::File::create(temp_file_path).expect("Failed to create temp file");
    //     writeln!(temp_file, "repositoryformatversion = 0").expect("Failed to write to temp file");
    //     writeln!(temp_file, "[remote \"origin\"]").expect("Failed to write to temp file");
    //     writeln!(temp_file, "   url = git@github.com:example/repo.git")
    //         .expect("Failed to write to temp file");
    //     writeln!(temp_file, "[branch \"main\"]").expect("Failed to write to temp file");
    //     writeln!(temp_file, "   remote = origin").expect("Failed to write to temp file");

    //     // Crea una instancia de GitConfig y lee la configuración desde el archivo temporal
    //     let result = GitConfig::read_git_config(temp_file_path);
    //     // Verifica que la lectura sea exitosa y que las secciones se hayan analizado correctamente
    //     assert!(result.is_ok());
    //     let git_config = result.unwrap();
    //     assert_eq!(git_config.core["repositoryformatversion"], "0");
    //     assert_eq!(
    //         git_config.remote_origin["url"],
    //         "git@github.com:example/repo.git"
    //     );
    //     assert_eq!(git_config.branch["remote"], "origin");

    //     // Limpia el archivo temporal después de las pruebas
    //     fs::remove_file(temp_file_path).expect("Failed to remove temp file");
    // }

    // #[test]
    // fn test_read_git_config_nonexistent_file() {
    //     let result = GitConfig::read_git_config("nonexistent_file.txt");
    //     assert!(result.is_ok());
    //     let git_config = result.unwrap();
    //     assert!(git_config.core.is_empty());
    //     assert!(git_config.remote_origin.is_empty());
    //     assert!(git_config.branch.is_empty());
    // }

    // #[test]
    // fn test_add_entry_core() {
    //     let mut git_config = GitConfig::new();
    //     git_config.add_entry("repositoryformatversion", "0", "core");

    //     assert_eq!(git_config.core["repositoryformatversion"], "0");
    //     assert!(git_config.remote_origin.is_empty());
    //     assert!(git_config.branch.is_empty());
    // }

    // #[test]
    // fn test_add_entry_remote_origin() {
    //     let mut git_config = GitConfig::new();
    //     git_config.add_entry("url", "git@github.com:example/repo.git", "remote.origin");

    //     assert!(git_config.core.is_empty());
    //     assert_eq!(
    //         git_config.remote_origin["url"],
    //         "git@github.com:example/repo.git"
    //     );
    //     assert!(git_config.branch.is_empty());
    // }

    // #[test]
    // fn test_add_entry_branch() {
    //     let mut git_config = GitConfig::new();
    //     git_config.add_entry("remote", "origin", "branch.main");

    //     assert!(git_config.core.is_empty());
    //     assert!(git_config.remote_origin.is_empty());
    //     assert_eq!(git_config.branch["remote"], "origin");
    // }

    // #[test]
    // fn test_add_entry_unknown_section() {
    //     let mut git_config = GitConfig::new();
    //     git_config.add_entry("key", "value", "unknown_section");

    //     assert!(git_config.core.is_empty());
    //     assert!(git_config.remote_origin.is_empty());
    //     assert!(git_config.branch.is_empty());
    // }

    // #[test]
    // fn test_get_value_core() {
    //     let mut git_config = GitConfig::new();
    //     git_config.add_entry("repositoryformatversion", "0", "core");

    //     assert_eq!(
    //         git_config.get_value("repositoryformatversion", "core"),
    //         Some(&"0".to_string())
    //     );
    //     assert_eq!(git_config.get_value("filemode", "core"), None);
    // }

    // #[test]
    // fn test_get_value_remote_origin() {
    //     let mut git_config = GitConfig::new();
    //     git_config.add_entry("url", "git@github.com:example/repo.git", "remote.origin");

    //     assert_eq!(
    //         git_config.get_value("url", "remote.origin"),
    //         Some(&"git@github.com:example/repo.git".to_string())
    //     );
    //     assert_eq!(git_config.get_value("fetch", "remote.origin"), None);
    // }

    // #[test]
    // fn test_get_value_branch() {
    //     let mut git_config = GitConfig::new();
    //     git_config.add_entry("remote", "origin", "branch.main");

    //     assert_eq!(
    //         git_config.get_value("remote", "branch.main"),
    //         Some(&"origin".to_string())
    //     );
    //     assert_eq!(git_config.get_value("merge", "branch.main"), None);
    // }

    // #[test]
    // fn test_get_value_unknown_section() {
    //     let git_config = GitConfig::new();
    //     assert_eq!(git_config.get_value("key", "unknown_section"), None);
    // }

    // #[test]
    // fn test_write_to_file() {
    //     let mut git_config = GitConfig::new();
    //     git_config.add_entry("url", "git@github.com:example/repo.git", "remote.origin");
    //     git_config.add_entry("remote", "origin", "branch.main");

    //     let file_path = "./test_files/config_2";
    //     let _ = git_config.write_to_file(file_path);

    //     let mut file_content = String::new();
    //     let mut file = File::open(file_path).expect("Could not open file");
    //     file.read_to_string(&mut file_content)
    //         .expect("Could not read file");

    //     let expected_content = "[remote \"origin\"]\n    \
    //                             url = git@github.com:example/repo.git\n\
    //                             [branch \"main\"]\n    \
    //                             remote = origin\n";
                                
    //     // Cleanup
    //     // fs::remove_file(file_path).expect("Could not remove file");
    //     assert_eq!(file_content, expected_content);
    // }

    // #[test]
    // fn test_read_git_config_from_file_1() {
    //     let file_path = "./test_files/config_3";
    //     let config = GitConfig::read_git_config(file_path).unwrap();
    //     assert!(config.core.is_empty());
    //     assert_eq!(
    //         config.remote_origin["url"],
    //         "git@github.com:example/repo.git");
    //     assert_eq!(config.branch["remote"], "origin");
    // }

    // #[test]
    // fn test_read_git_config_from_file_2() {
    //     let file_path = "./test_files/config_4";
    //     let config = GitConfig::read_git_config(file_path).unwrap();
    //     println!("{:?}", config);
    //     assert_eq!(config.core["repositoryformatversion"], "0");
    //     assert_eq!(config.core["filemode"], "true");
    //     assert_eq!(
    //         config.remote_origin["url"],
    //         "git@github.com:example/repo.git");
    //     assert_eq!(config.remote_origin["fetch"], "+refs/heads/*:refs/remotes/origin/*");
    //     assert_eq!(config.branch["remote"], "origin");
    //     assert_eq!(config.branch["merge"], "refs/heads/main");
    // }

    #[test]
    fn test_read_git_config_from_file_3() {
        let file_path = "./test_files/config_5";
        let config = GitConfig::_new_from_file(file_path).unwrap();
        println!("core => {:?}", config.core);
        println!("remote => {:?}", config.remote_origin);
        println!("branch => {:?}", config.branch);
        // println!("{:?}", config);
        // println!("HolaaAAAAA");
        assert!(true);
    }
}

// fn read_format_config(path: &str) -> Result<HashMap<String, HashMap<String, String>>, io::Error> {
//     let content = fs::read_to_string(path)?;
//     let mut result = HashMap::new();
//     let mut current_key = String::new();
//     let mut current_values = HashMap::new();

//     for line in content.lines() {
//         let line = line.trim();

//         if line.is_empty() || line.starts_with('#') {
//             continue;
//         }

//         let parts: Vec<&str> = line.splitn(2, '=').collect();

//         if parts.len() == 2 {
//             let key = parts[0].trim();

//             if key.starts_with('[') && key.ends_with(']') {
//                 if !current_key.is_empty() {
//                     result.insert(current_key.clone(), current_values.clone());
//                 }

//                 current_key = key[1..key.len() - 1].to_string();
//                 current_values.clear();
//             } else {
//                 let value = parts[1].trim();
//                 #[cfg(test)]

//     if !current_key.is_empty() {
//         result.insert(current_key, current_values);
//     }

//     Ok(result)
// }
