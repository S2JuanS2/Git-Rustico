use std::io::Read;

use crate::errors::GitError;

#[derive(Debug)]
pub struct ObjectEntry {
    pub object_type: ObjectType,
    pub object_length: usize,
}

#[derive(Debug)]
pub enum ObjectType {
    Commit,
    Tree,
    Blob,
    Tag,
    OfsDelta,
    RefDelta,
}



pub fn read_type_and_length(reader: &mut dyn Read) -> Result<ObjectEntry, GitError> {
    let mut type_and_length = [0u8; 1];
    if reader.read_exact(&mut type_and_length).is_err() {
        return Err(GitError::HeaderPackFileReadError);
    };

    let type_bits = (type_and_length[0] >> 4) & 0b00000111;
    let length_bits = (type_and_length[0] & 0b00001111) as usize;

    let object_type = create_object(type_bits)?;

    let mut object_length: usize = 0;
    for i in 0..length_bits {
        let mut byte = [0u8; 1];
        if reader.read_exact(&mut byte).is_err()
        {
            return Err(GitError::HeaderPackFileReadError);
        };
        object_length |= ((byte[0] as usize) & 0x7F) << (7 * i);
        if (byte[0] & 0x80) == 0 {
            break;
        }
    }

    Ok(ObjectEntry {
        object_type,
        object_length,
    })
}


fn create_object(type_bits :u8) -> Result<ObjectType, GitError>
{
    match type_bits {
        1 => Ok(ObjectType::Commit),
        2 => Ok(ObjectType::Tree),
        3 => Ok(ObjectType::Blob),
        4 => Ok(ObjectType::Tag),
        6 => Ok(ObjectType::OfsDelta),
        7 => Ok(ObjectType::RefDelta),
        _ => Err(GitError::InvalidObjectType),
    }
}