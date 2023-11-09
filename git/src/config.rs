//! # Estructura Config
//!
//! La estructura `Config` se utiliza para representar la configuración del cliente Git.
//! Esta configuración incluye información como el nombre y correo del usuario, y puede
//! ser personalizada mediante un archivo de configuración similar al archivo "gitconfig"
//! de Git.
//!
//! ## Nota
//!
//! La estructura `Config` se puede utilizar para gestionar la configuración del cliente Git,
//! incluyendo la personalización de datos como el nombre y correo del usuario.
//!
//! Asegúrate de configurar adecuadamente la estructura `Config` según las necesidades de tu
//! cliente Git y de utilizar los métodos proporcionados para acceder y modificar la
//! configuración según sea necesario.
//!

use std::{
    fmt,
    fs::File,
    io::{BufRead, BufReader},
};

use crate::{
    consts::*,
    util::validation::{valid_email, valid_ip, valid_port},
};
use crate::{errors::GitError, util::validation::valid_path_log};

type Operacion = fn(&str, &mut Config) -> Result<(), GitError>;

/// Estructura que representa la configuración del cliente Git.
#[derive(Debug)]
pub struct Config {
    pub name: String,
    pub email: String,
    pub path_log: String,
    pub ip: String,
    pub port: String,
    pub src: String,
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "Config:{{Name: {}, Email: {}, Log Path: {}, Ip: {}, Port: {}, Src: {}}}",
            self.name, self.email, self.path_log, self.ip, self.port, self.src
        )
    }
}

impl Config {
    /// Crea una nueva estructura `Config` a partir de un archivo de configuración
    /// ubicado en la ruta especificada en los argumentos recibidos por parametro.
    pub fn new(args: Vec<String>) -> Result<Config, GitError> {
        let path = parse_config_path(args)?;

        let mut config = Config {
            name: String::new(),
            email: String::new(),
            path_log: LOG_PATH_DEFAULT.to_string(),
            ip: IP_DEFAULT.to_string(),
            port: GIT_DAEMON_PORT.to_string(),
            src: SRC_DEFAULT.to_string(),
        };

        read_input(&path, &mut config, process_line)?;

        Ok(config)
    }
}

/// recibe los argumentos de entrada y devuelve el path del archivo de configuración o un error
/// en caso de que no haya la cantidad correcta de argumentos.
fn parse_config_path(args: Vec<String>) -> Result<String, GitError> {
    if args.len() > REQUIRED_ARG_COUNT {
        return Err(GitError::InvalidArgumentCountError);
    }

    if args.len() < REQUIRED_ARG_COUNT {
        return Err(GitError::MissingConfigPathError);
    }

    Ok(args[CONFIG_PATH_ARG_INDEX].clone())
}

/// Lee el archivo de configuración y procesa cada línea con la función `process`.
pub fn read_input(path: &str, config: &mut Config, process: Operacion) -> Result<(), GitError> {
    let file = open_file_for_reading(path)?;
    let mut reader: BufReader<File> = BufReader::new(file);
    let mut line = String::new();

    while let Ok(bytes_read) = reader.read_line(&mut line) {
        if bytes_read == 0 {
            break; // Fin del archivo
        }
        process(&line, config)?;
        line.clear();
    }
    Ok(())
}

/// Abre el archivo de configuración para lectura.
/// Devuelve un error en caso de que no se pueda abrir el archivo.
pub fn open_file_for_reading(path: &str) -> Result<File, GitError> {
    match File::open(path) {
        Ok(f) => Ok(f),
        Err(_) => Err(GitError::ConfigFileError),
    }
}

/// Procesa una línea del archivo de configuración y actualiza la configuración del cliente Git.
/// Devuelve un error en caso de que la línea no sea válida.
pub fn process_line(line: &str, config: &mut Config) -> Result<(), GitError> {
    let line = line.trim();
    let mut parts = line.split('=');
    let key = parts.next().ok_or(GitError::InvalidConfigFormatError)?;
    let value = parts.next().ok_or(GitError::InvalidConfigFormatError)?;
    // let key = parts.next().unwrap();
    // let value = parts.next().unwrap();

    match key {
        "name" => config.name = value.to_string(),
        "email" => config.email = valid_email(value)?,
        "path_log" => config.path_log = valid_path_log(value)?,
        "ip" => config.ip = valid_ip(value)?,
        "port" => config.port = valid_port(value)?,
        "src" => config.src = value.to_string(), //valid_directory_src(value)?,
        _ => return Err(GitError::InvalidConfigurationValueError),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_default_config() {
        let args = vec!["git".to_string(), "testfile".to_string()];
        let result = Config::new(args);
        let config = result.unwrap();
        assert_eq!(config.name, String::new());
        assert_eq!(config.email, String::new());
        assert_eq!(config.path_log, LOG_PATH_DEFAULT.to_string());
        assert_eq!(config.ip, IP_DEFAULT.to_string());
        assert_eq!(config.port, GIT_DAEMON_PORT.to_string());
    }

    #[test]
    fn test_parse_config_path_with_missing_args() {
        let args = vec![];
        let result = parse_config_path(args);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), GitError::MissingConfigPathError);
    }

    #[test]
    fn test_parse_config_path_with_too_many_args() {
        let args = vec!["git".to_string(), "path".to_string(), "extra".to_string()];
        let result = parse_config_path(args);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), GitError::InvalidArgumentCountError);
    }

    #[test]
    fn test_parse_config_path_with_valid_args() {
        let args = vec!["git".to_string(), "path".to_string()];
        let result = parse_config_path(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "path");
    }

    #[test]
    fn test_read_input_with_valid_path() {
        let mut config = Config::new(vec!["git".to_string(), "testfile".to_string()]).unwrap();
        let result = read_input("testfile", &mut config, process_line);
        assert!(result.is_ok());
    }

    #[test]
    fn test_open_file_for_reading_with_invalid_path() {
        let result = open_file_for_reading("invalid");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), GitError::ConfigFileError);
    }

    #[test]
    fn test_open_file_for_reading_with_valid_path() {
        let result = open_file_for_reading("testfile");
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_line_with_valid_key_value() {
        let mut config = Config::new(vec!["git".to_string(), "testfile".to_string()]).unwrap();
        let result = process_line("name=Test", &mut config);
        assert!(result.is_ok());
        assert_eq!(config.name, "Test");
    }

    #[test]
    fn test_process_line_with_invalid_key_value() {
        let mut config = Config::new(vec!["git".to_string(), "testfile".to_string()]).unwrap();
        let result = process_line("invalid=Test", &mut config);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            GitError::InvalidConfigurationValueError
        );
    }
}
