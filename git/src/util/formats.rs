use crate::errors::GitError;
use flate2::read::GzDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::{Read, Write};

/// Dado un contenido, genera el valor hash
/// ###Parametros:
/// 'content': contenido del que se creará el hash
pub fn hash_generate(content: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    let hash_commit = format!("{:x}", result);

    hash_commit
}

/// Dado un contenido lo comprime y lo guarda en un archivo
/// ###Parametros:
/// 'store': contenido que se comprimirá
/// 'file_object': archivo donde se guardará el contenido comprimido
pub fn compressor_object(store: String, mut file_object: File) -> Result<(), GitError> {
    let mut compressor = ZlibEncoder::new(Vec::new(), Compression::default());

    match compressor.write_all(store.as_bytes()) {
        Ok(_) => (),
        Err(_) => return Err(GitError::ReadFileError),
    }

    let compressed_bytes = match compressor.finish() {
        Ok(compressed_bytes) => compressed_bytes,
        Err(_) => return Err(GitError::ReadFileError),
    };

    match file_object.write_all(&compressed_bytes) {
        Ok(_) => (),
        Err(_) => return Err(GitError::ReadFileError),
    }

    Ok(())
}

/// Dado un contenido lo descomprime y lo guarda
/// ###Parametros:
/// 'content': contenido comprimido a descomprimir
pub fn decompression_object(compressed_data: &mut Vec<u8>) -> Result<String, GitError> {
    let mut decompression = GzDecoder::new(&compressed_data[..]);
    let mut content_string = String::new();

    match decompression.read_to_string(&mut content_string) {
        Ok(content_string) => content_string,
        Err(_) => return Err(GitError::DecompressionFailed),
    };

    Ok(content_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_generate_test() {
        let content = "Texto de prueba para calcular el hash";
        let expected_hash = "e449722207a52159f72f6ab7066775903dab9f79";

        let result_hash = hash_generate(content);

        assert_eq!(result_hash, expected_hash);
        assert_eq!(result_hash.len(), 40);
    }

    /*
    #[test]
    fn decompression_test() {
        // Datos comprimidos
        let mut compressed_data: Vec<u8> = vec![
            0x1F, 0x8B, 0x08, 0x08, 0xAB, 0xC4, 0x5A, 0x5A, 0x00, 0x03, 0x74, 0x65, 0x73, 0x74,
            0x00, 0x8B, 0xC8, 0xCE, 0xC9, 0xC9, 0x07, 0x00, 0x22, 0x10, 0x04, 0x03, 0x00, 0x00,
            0xFF, 0xFF, 0x0A, 0x00, 0x00, 0x00,
        ];

        let result = decompression_object(&mut compressed_data);

        // verifico que el contenido descomprimido es el esperado
        let content_string = String::from_utf8_lossy(&compressed_data);
        assert_eq!(content_string, "test");

        assert!(result.is_ok());
    }
    */
}
