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
//! let required_args = REQUIRED_ARG_COUNT;
//! println!("Número de argumentos requeridos por Git: {}", required_args);
//! ```
//!

// Argumentos requeridos por el programa Git.
// Use: cargo run -- <path config>
pub const REQUIRED_ARG_COUNT: usize = 2;

// El programa Git espera que el 1er argumento sea la ruta del archivo de configuración.
pub const CONFIG_PATH_ARG_INDEX: usize = 1;

// Path por defecto del archivo log.
pub const LOG_PATH_DEFAULT: &str = "./default.log";

// Path por defecto del src
pub const SRC_DEFAULT: &str = "client_root";

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

// Maximo valor de un puerto
pub const PORT_MAX: u16 = 65535;

// Minimo valor de un puerto
pub const PORT_MIN: u16 = 1024;

// Puerdo de git daemon
pub const GIT_DAEMON_PORT: u16 = 9418;

// Request de git-upload-pack
pub const GIT_UPLOAD_PACK: &str = "git-upload-pack";

// Request de git-receive-pack
pub const GIT_RECEIVE_PACK: &str = "git-receive-pack";

// Request de git-upload-archive
pub const GIT_UPLOAD_ARCHIVE: &str = "git-upload-archive";

pub const END_OF_STRING: &str = "\0";

// Tamaño del prefijo de longitud
pub const LENGTH_PREFIX_SIZE: usize = 4;

//
pub const FLUSH_PKT: &str = "0000";

pub const PKT_DONE: &str = "0009done\n";

pub const DONE: &str = "done";

pub const PKT_NAK: &str = "0008NAK\n";

pub const PACK_SIGNATURE: &str = "PACK";

pub const PACK_BYTES: [u8; 4] = [b'P', b'A', b'C', b'K'];

pub const SPACE: u8 = 32;

pub const NULL: u8 = 0;

pub const CONTINUE: &str = "continue";

pub const SHALLOW: &str = "shallow";

pub const HAVE: &str = "have";

pub const WANT: &str = "want";

pub const CAPABILITIES_FETCH: [&str; 1] = ["multi_ack"];

// Directorios
pub const GIT_DIR: &str = ".git";

pub const HEAD: &str = "HEAD";

pub const INITIAL_BRANCH: &str = "master";

pub const INDEX: &str = "index";

pub const HEAD_POINTER_REF: &str = "ref: refs/heads/";

pub const REF_HEADS: &str = "refs/heads";

pub const REFS: &str = "refs";

pub const REFS_HEADS: &str = "refs/heads";

pub const REFS_REMOTES: &str = "refs/remotes";

pub const REFS_TAGS: &str = "refs/tags";

pub const ORIGIN: &str = "origin";

pub const DIR_REFS: &str = "refs";

pub const DIR_OBJECTS: &str = "objects";

pub const CONTENT_EMPTY: &str = "";

// Objetos
pub const BLOB: &str = "blob";

pub const TREE: &str = "tree";

pub const COMMIT: &str = "commit";

pub const TAG: &str = "tag";

pub const ALL: &str = ".";

pub const DIRECTORY: &str = "40000";

pub const FILE: &str = "100644";

pub const PARENT_INITIAL: &str = "0000000000000000000000000000000000000000";

pub const VERSION_DEFAULT: u32 = 2;

pub const CONFIG_FILE: &str = "config";

pub const CONFIG_REMOTE_FETCH: &str = "+refs/heads/*:refs/remotes/origin/*";

pub const ZERO_ID: &str = "0000000000000000000000000000000000000000";