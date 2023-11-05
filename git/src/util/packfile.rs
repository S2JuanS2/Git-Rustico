use crate::{consts::{PACK_SIGNATURE, PACK_BYTES}, util::objects::read_type_and_length_from_vec, git_transport::{advertised::AdvertisedRefs, references::get_objects}};
use flate2::read::ZlibDecoder;
use std::io::{Read, Write};

use super::{errors::UtilError, objects::ObjectEntry, connections::send_bytes};

pub fn read_packfile_header(reader: &mut dyn Read) -> Result<u32, UtilError> {
    read_signature(reader)?;
    println!("Signature: {}", PACK_SIGNATURE);

    let version = read_version(reader)?;
    println!("Version: {}", version);

    let number_object = read_objects_contained(reader)?;
    println!("Number of objects: {}", number_object);
    Ok(number_object)
}

pub fn read_packfile_data(
    reader: &mut dyn Read,
    objects: usize,
) -> Result<Vec<(ObjectEntry, Vec<u8>)>, UtilError> {
    let mut information: Vec<(ObjectEntry, Vec<u8>)> = Vec::new();
    let mut buffer: Vec<u8> = Vec::new();
    match reader.read_to_end(&mut buffer) // Necesita refactorizar, si el packfile es muy grande habra problema
    {
        Ok(buffer) => buffer,
        Err(_) => return Err(UtilError::DataPackFiletReadObject),
    };
    let mut offset: usize = 0;

    for _ in 0..objects {
        let object_entry = read_type_and_length_from_vec(&buffer, &mut offset)?;
        println!("Object entry: {:?}", object_entry);
        let data: Vec<u8> = read_object_data(&buffer, &mut offset)?;

        if data.len() != object_entry.obj_length {
            return Err(UtilError::DataPackFiletReadObject);
        }
        information.push((object_entry, data));
    }
    Ok(information)
}

fn read_object_data(data: &[u8], offset: &mut usize) -> Result<Vec<u8>, UtilError> {
    let mut decompressed_data: Vec<u8> = Vec::new();

    let mut zlib_decoder: ZlibDecoder<&[u8]> = ZlibDecoder::new(&data[*offset..]);
    let n = match zlib_decoder.read_to_end(&mut decompressed_data) {
        Ok(n) => n,
        Err(_) => return Err(UtilError::ObjectDeserialization),
    };

    if n == 0 {
        return Err(UtilError::EmptyDecompressionError);
    }
    let bytes_read = zlib_decoder.total_in();
    *offset += bytes_read as usize;
    Ok(decompressed_data)
}

/// Lee y verifica la firma del encabezado del archivo PACKFILE a partir del lector proporcionado.
///
/// La firma es una secuencia de 4 bytes que debe coincidir con la firma esperada de Git.
///
/// # Argumentos
///
/// * `reader`: Un lector que implementa el trait `Read` para leer datos binarios.
///
/// # Retorno
///
/// Esta función no devuelve ningún valor útil en el éxito. En caso de éxito, significa que la
/// firma del encabezado del archivo PACKFILE se leyó correctamente y coincide con la firma
/// esperada. Si la firma no coincide o se produce un error de lectura, devuelve un error
/// `UtilError`.
fn read_signature(reader: &mut dyn Read) -> Result<(), UtilError> {
    let mut buffer = [0u8; 4];
    if reader.read_exact(&mut buffer).is_err() {
        return Err(UtilError::HeaderPackFileReadSignature);
    };

    if buffer != PACK_SIGNATURE.as_bytes() {
        return Err(UtilError::HeaderPackFileReadSignature);
    }
    Ok(())
}

/// Lee y verifica la versión del encabezado del archivo PACKFILE a partir del lector proporcionado.
///
/// La versión es un número de 4 bytes que debe ser 2 o 3 para ser compatible con el formato del archivo PACKFILE de Git.
///
/// # Argumentos
///
/// * `reader`: Un lector que implementa el trait `Read` para leer datos binarios.
///
/// # Retorno
///
/// En caso de éxito, devuelve el valor de la versión del archivo PACKFILE leído. Si la versión
/// no es 2 o 3, o se produce un error de lectura, se devuelve un error `UtilError`.
fn read_version(reader: &mut dyn Read) -> Result<u32, UtilError> {
    let mut buffer = [0u8; 4];
    if reader.read_exact(&mut buffer).is_err() {
        return Err(UtilError::HeaderPackFileReadVersion);
    };

    let version: u32 = u32::from_be_bytes(buffer);
    if version != 2 && version != 3 {
        return Err(UtilError::HeaderPackFileReadVersion);
    }

    Ok(version)
}

/// Lee y verifica el número de objetos contenidos en el encabezado del archivo PACKFILE a partir del lector proporcionado.
///
/// Este número es un valor de 4 bytes que representa la cantidad de objetos en el archivo PACKFILE.
///
/// # Argumentos
///
/// * `reader`: Un lector que implementa el trait `Read` para leer datos binarios.
///
/// # Retorno
///
/// En caso de éxito, devuelve el número de objetos contenidos en el archivo PACKFILE leído. En caso de error de lectura, se devuelve un error `UtilError`.
fn read_objects_contained(reader: &mut dyn Read) -> Result<u32, UtilError> {
    let mut buffer = [0; 4];
    if reader.read_exact(&mut buffer).is_err() {
        return Err(UtilError::HeaderPackFileReadNumberObjects);
    };

    let value = u32::from_be_bytes(buffer);

    Ok(value)
}

pub fn send_packfile(writer: &mut dyn Write, advertised: &AdvertisedRefs, path_repo: &str) -> Result<(), UtilError> {
    // Envio signature
    send_bytes(writer, &PACK_BYTES, UtilError::SendSignaturePackfile)?;

    // Envio version
    send_bytes(writer, &advertised.version.to_be_bytes(), UtilError::SendSignaturePackfile)?;
    
    // Envio numero de objetos
    let objects = match get_objects(path_repo, &advertised.references)
    {
        Ok(objects) => objects,
        Err(_) => return Err(UtilError::GetObjectsPackfile),
    };
    let number_objects = objects.len() as u32;
    send_bytes(writer, &number_objects.to_be_bytes(), UtilError::SendSignaturePackfile)?;
    
    
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Cursor};

    #[test]
    fn test_read_signature_valid_signature() -> Result<(), UtilError> {
        let data: [u8; 4] = [b'P', b'A', b'C', b'K']; // Firma válida "PACK"
        let mut cursor = Cursor::new(&data);
        read_signature(&mut cursor)?;

        Ok(())
    }

    #[test]
    fn test_read_signature_invalid_data() {
        let data: [u8; 3] = [b'P', b'A', b'C']; // Datos de longitud incorrecta
        let mut cursor = Cursor::new(&data);
        let result = read_signature(&mut cursor);

        assert!(result.is_err());
    }

    #[test]
    fn test_read_signature_invalid_signature() {
        let data: [u8; 4] = [b'P', b'A', b'X', b'K']; // Firma incorrecta
        let mut cursor = Cursor::new(&data);
        let result = read_signature(&mut cursor);

        assert!(result.is_err());
    }

    #[test]
    fn test_read_signature_io_error() {
        let mut invalid_reader = io::empty(); // Un lector vacío provocará un error
        let result = read_signature(&mut invalid_reader);

        assert!(result.is_err());
    }

    #[test]
    fn test_read_version_valid_version_2() -> Result<(), UtilError> {
        let data: [u8; 4] = [0, 0, 0, 2]; // Versión válida 2
        let mut cursor = Cursor::new(&data);
        let version = read_version(&mut cursor)?;

        assert_eq!(version, 2);
        Ok(())
    }

    #[test]
    fn test_read_version_valid_version_3() -> Result<(), UtilError> {
        let data: [u8; 4] = [0, 0, 0, 3]; // Versión válida 3
        let mut cursor = Cursor::new(&data);
        let version = read_version(&mut cursor)?;

        assert_eq!(version, 3);
        Ok(())
    }

    #[test]
    fn test_read_version_invalid_data() {
        let data: [u8; 3] = [0, 0, 2]; // Datos de longitud incorrecta
        let mut cursor = Cursor::new(&data);
        let result = read_version(&mut cursor);

        assert!(result.is_err());
    }

    #[test]
    fn test_read_version_invalid_version() {
        let data: [u8; 4] = [0, 0, 0, 4]; // Versión inválida (no es 2 ni 3)
        let mut cursor = Cursor::new(&data);
        let result = read_version(&mut cursor);

        assert!(result.is_err());
    }

    #[test]
    fn test_read_version_io_error() {
        let mut invalid_reader = io::empty(); // Un lector vacío provocará un error
        let result = read_version(&mut invalid_reader);

        assert!(result.is_err());
    }

    #[test]
    fn test_read_objects_contained_valid() -> Result<(), UtilError> {
        let data: [u8; 4] = [0, 0, 0, 42]; // Número de objetos: 42
        let mut cursor = Cursor::new(&data);
        let num_objects = read_objects_contained(&mut cursor)?;

        assert_eq!(num_objects, 42);
        Ok(())
    }

    #[test]
    fn test_read_objects_contained_invalid_data() {
        let data: [u8; 3] = [0, 0, 42]; // Datos de longitud incorrecta
        let mut cursor = Cursor::new(&data);
        let result = read_objects_contained(&mut cursor);

        assert!(result.is_err());
    }

    #[test]
    fn test_read_objects_contained_io_error() {
        let mut invalid_reader = io::empty(); // Un lector vacío provocará un error
        let result = read_objects_contained(&mut invalid_reader);

        assert!(result.is_err());
    }
}
