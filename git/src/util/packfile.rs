use super::objects::read_type_and_length;
use crate::{consts::PACK_SIGNATURE, errors::GitError};
use std::io::Read;
// use std::fs::File;
// use flate2::read::ZlibDecoder;

// fn process_packfile(file_path: &str) -> Result<(), GitError> {
//     let mut packfile = match File::open(file_path)
//     {
//         Ok(file) => file,
//         Err(e) => {
//             println!("Error: {}", e);
//             return Err(GitError::GenericError);
//         }
//     };

//     let decompressed = ZlibDecoder::new(&mut packfile);
//     Ok(())
// }

pub fn read_packfile_header(reader: &mut dyn Read) -> Result<(), GitError> {
    read_signature(reader)?;
    println!("Signature: {}", PACK_SIGNATURE);

    let version = read_version(reader)?;
    println!("Version: {}", version);

    let number_object = read_objects_contained(reader)?;
    println!("Number of objects: {}", number_object);

    for _ in 0..number_object {
        let object_entry = read_type_and_length(reader)?;
        let mut buffer = vec![0u8; object_entry.obj_length as usize];
        if reader.read_exact(&mut buffer).is_err() {
            return Err(GitError::HeaderPackFileReadError);
        };
        println!("Object entry: {:?}", object_entry);
        println!("Object data len: {:?}", buffer.len());
        // let m = String::from_utf8(buffer.to_vec()).expect("No se pudo convertir a String");
        // println!("Lectura exitosa: {:?}", m);
        println!("Object data: {:?}", buffer)
    }
    Ok(())
}

pub fn read_packfile_data(reader: &mut dyn Read) -> Result<(), GitError> {
    let mut buffer = [0; 4096]; // Tamaño del búfer de lectura
    match reader.read(&mut buffer) {
        Ok(_) => {
            let m = String::from_utf8(buffer.to_vec()).expect("No se pudo convertir a String");
            println!("Lectura exitosa: {:?}", m);
        }
        Err(e) => {
            println!("Error: {}", e);
            return Err(GitError::GenericError);
        }
    };
    Ok(())
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
/// `GitError`.
fn read_signature(reader: &mut dyn Read) -> Result<(), GitError> {
    let mut buffer = [0u8; 4];
    if reader.read_exact(&mut buffer).is_err() {
        return Err(GitError::HeaderPackFileReadError);
    };

    if buffer != PACK_SIGNATURE.as_bytes() {
        return Err(GitError::HeaderPackFileReadError);
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
/// no es 2 o 3, o se produce un error de lectura, se devuelve un error `GitError`.
fn read_version(reader: &mut dyn Read) -> Result<u32, GitError> {
    let mut buffer = [0u8; 4];
    if reader.read_exact(&mut buffer).is_err() {
        return Err(GitError::HeaderPackFileReadError);
    };

    let version: u32 = u32::from_be_bytes(buffer);
    if version != 2 && version != 3 {
        return Err(GitError::HeaderPackFileReadError);
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
/// En caso de éxito, devuelve el número de objetos contenidos en el archivo PACKFILE leído. En caso de error de lectura, se devuelve un error `GitError`.
fn read_objects_contained(reader: &mut dyn Read) -> Result<u32, GitError> {
    let mut buffer = [0; 4];
    if reader.read_exact(&mut buffer).is_err() {
        return Err(GitError::HeaderPackFileReadError);
    };

    let value = u32::from_be_bytes(buffer);

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Cursor};

    #[test]
    fn test_read_signature_valid_signature() -> Result<(), GitError> {
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
    fn test_read_version_valid_version_2() -> Result<(), GitError> {
        let data: [u8; 4] = [0, 0, 0, 2]; // Versión válida 2
        let mut cursor = Cursor::new(&data);
        let version = read_version(&mut cursor)?;

        assert_eq!(version, 2);
        Ok(())
    }

    #[test]
    fn test_read_version_valid_version_3() -> Result<(), GitError> {
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
    fn test_read_objects_contained_valid() -> Result<(), GitError> {
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
