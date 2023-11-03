use std::{net::TcpStream, fs};

use super::{advertised::AdvertisedRefs, errors::UtilError, connections::send_message, pkt_line};

/// Realiza un proceso de descubrimiento de referencias (refs) enviando un mensaje al servidor
/// a través del socket proporcionado, y luego procesa las líneas recibidas para clasificarlas
/// en una lista de AdvertisedRefs.
///
/// # Argumentos
/// - `socket`: Un TcpStream que representa la conexión con el servidor.
/// - `message`: Un mensaje que se enviará al servidor.
///
/// # Retorno
/// Un Result que contiene un vector de AdvertisedRefs si la operación fue exitosa,
/// o un error de UtilError en caso contrario.
pub fn reference_discovery(
    socket: &mut TcpStream,
    message: String,
) -> Result<Vec<AdvertisedRefs>, UtilError> {
    send_message(socket, message, UtilError::ReferenceDiscovey)?;
    let lines = pkt_line::read(socket)?;
    AdvertisedRefs::classify_vec(&lines)
}


// A mejorar
// El packet-ref deberia eliminar esto
pub fn list_references(repo_path: &str) -> Result<Vec<String>, UtilError> {
    let mut references: Vec<String> = Vec::new();

    let refs_dir = format!("{}/.git/refs", repo_path);

    if let Ok(entries) = fs::read_dir(refs_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.starts_with("heads/") || file_name.starts_with("tags/") {
                        references.push(file_name.to_string());
                    }
                }
            }
        }
    }

    Ok(references)
}