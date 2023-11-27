use crate::util::errors::UtilError;
extern crate flate2;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::{Read, Write};

/// Dado un contenido en bytes, genera el valor hash
/// ###Parametros:
/// 'content': contenido del que se creará el hash
pub fn hash_generate_with_bytes(content: Vec<u8>) -> String {
    let mut hasher = Sha1::new();
    hasher.update(content);
    let result = hasher.finalize();
    let hash_commit = format!("{:x}", result);

    hash_commit
}

/// Dado un contenido en caracteres, genera el valor hash
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
pub fn compressor_object_with_bytes(store: Vec<u8>, mut file_object: File) -> Result<(), UtilError> {
    let mut compressor = ZlibEncoder::new(Vec::new(), Compression::default());

    match compressor.write_all(&store) {
        Ok(_) => (),
        Err(_) => return Err(UtilError::ReadFileError),
    }

    let compressed_bytes = match compressor.finish() {
        Ok(compressed_bytes) => compressed_bytes,
        Err(_) => return Err(UtilError::ReadFileError),
    };

    match file_object.write_all(&compressed_bytes) {
        Ok(_) => (),
        Err(_) => return Err(UtilError::ReadFileError),
    }

    Ok(())
}

/// Dado un contenido lo comprime y lo guarda en un archivo
/// ###Parametros:
/// 'store': contenido que se comprimirá
/// 'file_object': archivo donde se guardará el contenido comprimido
pub fn compressor_object(store: String, mut file_object: File) -> Result<(), UtilError> {
    let mut compressor = ZlibEncoder::new(Vec::new(), Compression::default());

    match compressor.write_all(store.as_bytes()) {
        Ok(_) => (),
        Err(_) => return Err(UtilError::ReadFileError),
    }

    let compressed_bytes = match compressor.finish() {
        Ok(compressed_bytes) => compressed_bytes,
        Err(_) => return Err(UtilError::ReadFileError),
    };

    match file_object.write_all(&compressed_bytes) {
        Ok(_) => (),
        Err(_) => return Err(UtilError::ReadFileError),
    }

    Ok(())
}

/// Dado un directorio lo descomprime y lo guarda
/// ###Parametros:
/// 'content': directorio del archivo comprimido a descomprimir
pub fn decompression_object(path: &str) -> Result<Vec<u8>, UtilError> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return Err(UtilError::OpenFileError),
    };

    let mut reader = ZlibDecoder::new(file);

    let mut uncompressed_content = Vec::new();
    if reader.read_to_end(&mut uncompressed_content).is_err() {
        return Err(UtilError::ReadFileError);
    };

    Ok(uncompressed_content)
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

    #[test]
    fn test_compressor_and_decompression_objects() {
        // Contenido de prueba
        let test_content = "Hola, este es un contenido de prueba";
        // Nombre de archivo de prueba
        let test_file = "test_file";

        // Llamar a la función de compresión
        let file_for_compression =
            File::create(test_file).expect("Falló al crear el archivo de prueba");
        match compressor_object(test_content.to_string(), file_for_compression) {
            Ok(_) => (),
            Err(err) => {
                panic!("Falló la compresión: {:?}", err);
            }
        }
        // Llamar a la función de descompresión
        match decompression_object(test_file) {
            Ok(result) => {
                // El contenido descomprimido debe ser igual al contenido original
                assert_eq!(String::from_utf8_lossy(&result), test_content);
            }
            Err(err) => {
                panic!("Falló en la descompresión: {:?}", err);
            }
        }
        // Se borra el archivo de prueba
        std::fs::remove_file(test_file).expect("Falló al remover el archivo de prueba");
    }
}
