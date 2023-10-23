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

fn read_objects_contained(reader: &mut dyn Read) -> Result<u32, GitError> {
    let mut buffer = [0; 4];
    if reader.read_exact(&mut buffer).is_err() {
        return Err(GitError::HeaderPackFileReadError);
    };

    let value = u32::from_be_bytes(buffer);

    Ok(value)
}
