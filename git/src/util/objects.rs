use std::io::Read;

use crate::errors::GitError;

/// Estructura que representa una entrada de objeto en el sistema de control de versiones Git.
///
/// Una entrada de objeto almacena información sobre el tipo y la longitud de un objeto Git.
/// Esto es esencial para identificar y gestionar objetos almacenados en un repositorio Git.

/// - `object_type`: El tipo de objeto, representado mediante un valor de la enumeración `ObjectType`.
/// - `object_length`: La longitud o tamaño del objeto en bytes.
///
#[derive(Debug)]
pub struct ObjectEntry {
    pub object_type: ObjectType,
    pub object_length: usize,
}

/// Enumeración que representa los tipos de objetos Git.
///
/// Cada variante de esta enumeración corresponde a un tipo de objeto en el sistema de control de versiones Git.
/// Estos tipos de objetos son utilizados para identificar y gestionar los distintos tipos de datos almacenados en un repositorio Git.

/// - `Commit`: Objeto de tipo Commit, que representa un commit en Git.
/// - `Tree`: Objeto de tipo Tree, que representa un árbol de directorios en Git.
/// - `Blob`: Objeto de tipo Blob, que representa un archivo o contenido binario en Git.
/// - `Tag`: Objeto de tipo Tag, que representa una etiqueta en Git.
/// - `OfsDelta`: Objeto de tipo OfsDelta, que representa un objeto delta relativo a una posición en un paquete.
/// - `RefDelta`: Objeto de tipo RefDelta, que representa un objeto delta referenciado en un paquete.
///
#[derive(Debug, PartialEq)]
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

    let object_type: ObjectType = create_object(type_bits)?;

    let object_length: usize = read_variable_length(reader, length_bits)?;

    Ok(ObjectEntry {
        object_type,
        object_length,
    })
}

fn read_variable_length(reader: &mut dyn Read, length_bits: usize) -> Result<usize, GitError> {
    let mut object_length: usize = 0;
    for i in 0..length_bits {
        let mut byte = [0u8; 1];
        if reader.read_exact(&mut byte).is_err() {
            return Err(GitError::HeaderPackFileReadError);
        };
        object_length |= ((byte[0] as usize) & 0x7F) << (7 * i);
        if (byte[0] & 0x80) == 0 {
            break;
        }
    }
    Ok(object_length)
}

/// Crea un objeto `ObjectType` a partir de un valor de tipo de objeto representado por `type_bits`.
///
/// # Argumentos
///
/// * `type_bits`: Un valor de 8 bits que representa el tipo de objeto.
///
/// # Retorno
///
/// * `Ok(ObjectType)`: Si el valor de `type_bits` corresponde a un tipo de objeto válido.
/// * `Err(GitError)`: Si el valor de `type_bits` no corresponde a un tipo de objeto válido y genera un error `GitError::InvalidObjectType`.
///
fn create_object(type_bits: u8) -> Result<ObjectType, GitError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::GitError;

    #[test]
    fn test_create_object_commit() {
        let object_type = create_object(1);
        assert_eq!(object_type, Ok(ObjectType::Commit));
    }

    #[test]
    fn test_create_object_tree() {
        let object_type = create_object(2);
        assert_eq!(object_type, Ok(ObjectType::Tree));
    }

    #[test]
    fn test_create_object_blob() {
        let object_type = create_object(3);
        assert_eq!(object_type, Ok(ObjectType::Blob));
    }

    #[test]
    fn test_create_object_tag() {
        let object_type = create_object(4);
        assert_eq!(object_type, Ok(ObjectType::Tag));
    }

    #[test]
    fn test_create_object_ofs_delta() {
        let object_type = create_object(6);
        assert_eq!(object_type, Ok(ObjectType::OfsDelta));
    }

    #[test]
    fn test_create_object_ref_delta() {
        let object_type = create_object(7);
        assert_eq!(object_type, Ok(ObjectType::RefDelta));
    }

    #[test]
    fn test_create_object_invalid_type_5() {
        let object_type = create_object(5); // Tipo inválido
        assert_eq!(object_type, Err(GitError::InvalidObjectType));
    }

    #[test]
    fn test_create_object_invalid_type_0() {
        let object_type = create_object(0); // Tipo inválido
        assert_eq!(object_type, Err(GitError::InvalidObjectType));
    }
}
