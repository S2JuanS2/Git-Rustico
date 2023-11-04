use std::{net::TcpStream, fs};

use crate::util::advertised::AdvertisedRefs;

use super::{errors::UtilError, connections::send_message, pkt_line};


#[derive(Debug)]
pub struct Reference {
    hash: String,
    name: String,
}

impl Reference {
    pub fn new(hash: String, name: String) -> Reference {
        Reference {
            hash,
            name,
        }
    }

    pub fn get_hash(&self) -> &String {
        &self.hash
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }
}

/// Realiza un proceso de descubrimiento de referencias (refs) enviando un mensaje al servidor
/// a través del socket proporcionado, y luego procesa las líneas recibidas para clasificarlas
/// en una lista de AdvertisedRefLine.
///
/// # Argumentos
/// - `socket`: Un TcpStream que representa la conexión con el servidor.
/// - `message`: Un mensaje que se enviará al servidor.
///
/// # Retorno
/// Un Result que contiene un vector de AdvertisedRefLine si la operación fue exitosa,
/// o un error de UtilError en caso contrario.
pub fn reference_discovery(
    socket: &mut TcpStream,
    message: String,
) -> Result<AdvertisedRefs, UtilError> {
    send_message(socket, message, UtilError::ReferenceDiscovey)?;
    let lines = pkt_line::read(socket)?;
    println!("lines: {:?}", lines);
    AdvertisedRefs::new(&lines)
    // let filtered_lines: Vec<Vec<u8>> = lines.into_iter().skip(1).collect();
    // Ok((refs, filtered_lines))
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