use std::io::Read;

use crate::{consts::PACK_SIGNATURE, errors::GitError};


pub fn read_packfile_header(reader: &mut dyn Read) -> Result<(), GitError> {
    read_signature(reader)?;
    println!("Signature: {}", PACK_SIGNATURE);
    let version = read_version(reader)?;
    println!("Version: {}", version);
    let number_object = read_objects_contained(reader)?;
    println!("Number of objects: {}", number_object);
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
