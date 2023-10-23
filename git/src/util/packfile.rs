use std::io::Read;

use crate::{errors::GitError, consts::PACK_SIGNATURE};

pub fn read_pack_header(reader: &mut dyn Read) -> Result<(), GitError>
{
    read_signature(reader)?;
    let version = read_version(reader)?;
    println!("Version: {}", version);
    let number_object = read_objects_contained(reader)?;
    println!("Number of objects: {}", number_object);
    Ok(())
}

fn read_signature(reader: &mut dyn Read) -> Result<(), GitError>
{
    let mut buffer = [0u8; 4];
    if reader.read_exact(&mut buffer).is_err() {
        return Err(GitError::HeaderPackFileReadError);
    };
    
    if &buffer != PACK_SIGNATURE.as_bytes() {
        return Err(GitError::HeaderPackFileReadError);
    }
    Ok(())
}

fn read_version(reader: &mut dyn Read) -> Result<u8, GitError> {
    let mut buffer = [0u8; 4];
    if reader.read_exact(&mut buffer).is_err() {
        return Err(GitError::HeaderPackFileReadError);
    };
    
    let version = buffer[0];
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