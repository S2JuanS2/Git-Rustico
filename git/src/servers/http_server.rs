//! Módulo principal del servidor HTTP.
//!
//! Este módulo contiene submódulos para manejar diferentes aspectos del servidor HTTP.
//! Incluye la gestión de solicitudes HTTP, utilidades auxiliares y manejo de conexiones HTTP.

pub mod http_request;

pub mod utils;

pub mod http_connection;

pub mod pr;

pub mod status_code;

pub mod http_body;

pub mod features_pr;

pub mod pr_registry;

pub mod method;

pub mod model;