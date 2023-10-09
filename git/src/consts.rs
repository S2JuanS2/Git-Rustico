//! # Módulo Consts
//!
//! El módulo `consts` contiene constantes utilizadas en todo el proyecto de Git.
//! Estas constantes son valores que se utilizan en múltiples partes del código para mantener
//! la coherencia y facilitar la personalización.
//!
//! ## Ejemplos
//!
//! ```
//! use git::consts::*;
//!
//! let max_attempts = CONNECTION_RETRY_MAX_ATTEMPTS;
//! println!("Número máximo de intentos de conexión: {}", max_attempts);
//! ```
//!

// Argumentos requeridos por el programa Git.
// Use: cargo run -- <path config>
pub const REQUIRED_ARG_COUNT: usize = 2;

// El programa Git espera que el 1er argumento sea la ruta del archivo de configuración.
pub const CONFIG_PATH_ARG_INDEX: usize = 1;

// Path por defecto del archivo log.
pub const DEFAULT_LOG_PATH: &str = "./default.log";
