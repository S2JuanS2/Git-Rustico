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
};

use crate::{consts::*, util::validation::{is_valid_ip, valid_path_log, valid_email}};
use crate::errors::GitError;

type Operacion = fn(&str, &mut Config) -> Result<(), GitError>;

/// Estructura que representa la configuración del cliente Git.
#[derive(Debug)]
pub struct Config {
    pub name: String,
    pub email: String,
    pub path_log: String,
    pub ip: String,
    pub port: String,
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Name: {}\nEmail: {}\nLog Path: {}\nIp: {}\nPort: {}\n",
            self.name, self.email, self.path_log, self.ip, self.port
        )
    }
}

impl Config {
    pub fn new() -> Result<Config, GitError> {
        let path = parse_config_path()?;

        let mut config = Config {
            name: String::new(),
            email: String::new(),
            path_log: LOG_PATH_DEFAULT.to_string(),
            ip: IP_DEFAULT.to_string(),
            port: PORT_DEFAULT.to_string(),
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
        "ip" => config.ip = is_valid_ip(value.to_string())?,
        "port" => config.port = value.to_string(),
        _ => return Err(GitError::InvalidConfigurationValueError),
    }
    Ok(())
}
