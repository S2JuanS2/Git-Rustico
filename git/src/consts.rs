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
pub const LOG_PATH_DEFAULT: &str = "./default.log";

// IP por default
pub const IP_DEFAULT: &str = "127.0.0.1";

// Port por default
pub const PORT_DEFAULT: &str = "50389";

// Numero de octetos de una IPV4
pub const IPV4_SECTIONS: usize = 4;

// Maximo valor de un octeto de IPV4
pub const IPV4_MAX: u16 = 255;

// Numero de octetos de una IPV6
pub const IPV6_SECTIONS: usize = 8;

// Maximo valor de una seccion de IPV6
pub const IPV6_MAX: u16 = 65535;

// Longitud de las secciones de IPV6
pub const IPV4_SECTION_LENGTH: usize = 4;
