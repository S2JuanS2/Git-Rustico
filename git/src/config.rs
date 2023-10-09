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
    env, fmt,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::consts::*;
use crate::errors::GitError;

type Operacion = fn(&str, &mut Config) -> Result<(), GitError>;

/// Estructura que representa la configuración del cliente Git.
#[derive(Debug)]
pub struct Config {
    pub name: String,
    pub email: String,
    pub path_log: String,
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Name: {}\nEmail: {}\nLog Path: {}\n",
            self.name, self.email, self.path_log
        )
    }
}

impl Config {
    pub fn new() -> Result<Config, GitError> {
        let path = parse_config_path()?;

        let mut config = Config {
            name: String::new(),
            email: String::new(),
            path_log: DEFAULT_LOG_PATH.to_string(),
        };

        read_input(&path, &mut config, process_line)?;

        Ok(config)
    }
}

fn parse_config_path() -> Result<String, GitError> {
    let args: Vec<String> = env::args().collect();
    if args.len() > REQUIRED_ARG_COUNT {
        return Err(GitError::InvalidArgumentCountError);
    }

    if args.len() < REQUIRED_ARG_COUNT {
        return Err(GitError::MissingConfigPathError);
    }

    Ok(args[CONFIG_PATH_ARG_INDEX].clone())
}

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

pub fn open_file_for_reading(path: &str) -> Result<File, GitError> {
    match File::open(path) {
        Ok(f) => Ok(f),
        Err(_) => Err(GitError::ConfigFileError),
    }
}

pub fn process_line(line: &str, config: &mut Config) -> Result<(), GitError> {
    let line = line.trim();
    let mut parts = line.split('=');
    let key = parts.next().unwrap();
    let value = parts.next().unwrap();

    match key {
        "name" => config.name = value.to_string(),
        "email" => config.email = valid_email(value)?,
        "path_log" => config.path_log = valid_path_log(value)?,
        _ => return Err(GitError::InvalidConfigurationValueError),
    }
    Ok(())
}

fn valid_path_log(file_path: &str) -> Result<String, GitError> {
    // Obtener el directorio padre del path del archivo
    if let Some(parent_dir) = Path::new(file_path).parent() {
        if parent_dir.exists() && parent_dir.is_dir() {
            Ok(file_path.to_string())
        } else {
            Err(GitError::InvalidLogDirectoryError)
        }
    } else {
        Err(GitError::InvalidLogDirectoryError)
    }
}

fn valid_email(email: &str) -> Result<String, GitError> {
    let parts: Vec<&str> = email.split('@').collect();

    if parts.len() != 2 {
        return Err(GitError::InvalidUserMailError);
    }

    let local_part = parts[0];
    let domain_part = parts[1];

    if local_part.is_empty() || domain_part.is_empty() {
        return Err(GitError::InvalidUserMailError);
    }

    // Verificar que el local part no contenga caracteres inválidos
    for c in local_part.chars() {
        if !c.is_alphanumeric() && c != '.' && c != '-' && c != '_' {
            return Err(GitError::InvalidUserMailError);
        }
    }

    // Verificar que el dominio tenga al menos un punto y no contenga caracteres inválidos
    let domain_parts: Vec<&str> = domain_part.split('.').collect();
    if domain_parts.len() < 2 {
        return Err(GitError::InvalidUserMailError);
    }

    for part in domain_parts {
        for c in part.chars() {
            if !c.is_alphanumeric() && c != '-' {
                return Err(GitError::InvalidUserMailError);
            }
        }
    }

    Ok(email.to_string())
}
