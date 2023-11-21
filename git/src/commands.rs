//! # Módulo de Comandos Git
//!
//! El módulo `commands` contiene submódulos que representan varios comandos de Git y sus implementaciones. Cada submódulo encapsula la lógica y funcionalidad de un comando Git específico, lo que facilita la administración y la ampliación del código.
//!
//! ## Submódulos
//!
//! - [`init.rs`](init/index.html): Contiene la lógica del comando `git init`, que inicializa un nuevo repositorio Git.
//! - [`push.rs`](push/index.html): Representa el comando `git push`, que se utiliza para enviar los cambios locales a un repositorio remoto.
//! - ...

/// Importa submódulos específicos para los comandos Git.
pub mod add;
pub mod branch;
pub mod cat_file;
pub mod checkout;
pub mod clone;
pub mod commit;
pub mod config;
pub mod errors;
pub mod fetch;
pub mod hash_object;
pub mod init;
pub mod log;
pub mod ls_files;
pub mod ls_tree;
pub mod merge;
pub mod pull;
pub mod push;
pub mod remote;
pub mod rm;
pub mod status;
