use crate::consts::*;
use crate::errors::GitError;
use std::io::Read;

/// Estructura que representa una entrada de objeto en el sistema de control de versiones Git.
///
/// Una entrada de objeto almacena información sobre el tipo y la longitud de un objeto Git.
/// Esto es esencial para identificar y gestionar objetos almacenados en un repositorio Git.

/// - `object_type`: El tipo de objeto, representado mediante un valor de la enumeración `ObjectType`.
/// - `object_length`: La longitud o tamaño del objeto en bytes.
///
#[derive(Debug, PartialEq)]
pub struct ObjectEntry {
    pub obj_type: ObjectType,
    pub obj_length: usize,
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
    let mut buffer = [0u8; 1];
    if reader.read_exact(&mut buffer).is_err() {
        return Err(GitError::HeaderPackFileReadError);
    };
    println!("(read_type_and_length)Buffer: {:?}", buffer);
    let byte = buffer[0];

    let obj_type: ObjectType = create_object_bits(byte)?;
    let length = read_size_encoded_length(reader, byte)?;

    Ok(ObjectEntry {
        obj_type,
        obj_length: length,
    })
}

pub fn read_type_and_length_from_vec(
    data: &[u8],
    offset: &mut usize,
) -> Result<ObjectEntry, GitError> {
    let byte = data[*offset];
    *offset += 1;
    let obj_type: ObjectType = create_object_bits(byte)?;
    let length = read_size_encoded_length_from_vec(data, byte, offset)?;

    Ok(ObjectEntry {
        obj_type,
        obj_length: length,
    })
}

fn read_size_encoded_length_from_vec(
    data: &[u8],
    byte: u8,
    offset: &mut usize,
) -> Result<usize, GitError> {
    let mut length_bits = (byte & 0b00001111) as usize;
    if (byte & 0b10000000) == 0 {
        return Ok(length_bits); // Se gasto un bit para el tipo
    }
    let mut shift: usize = 4;

    loop {
        let byte = data[*offset];
        *offset += 1;

        let seven_bits = (byte & 0b01111111) as usize;
        length_bits |= seven_bits << shift;
        if (byte & 0x80) == 0 {
            break;
        }

        shift += 7;
    }
    Ok(length_bits)
}

fn read_size_encoded_length(reader: &mut dyn Read, byte: u8) -> Result<usize, GitError> {
    let mut length_bits = (byte & 0b00001111) as usize;
    if (byte & 0b10000000) == 0 {
        return Ok(length_bits); // Se gasto un bit para el tipo
    }

    println!("(MSB)Length firts: {:?}", length_bits);
    let mut shift: usize = 4;

    loop {
        let mut byte = [0u8; 1];
        if reader.read_exact(&mut byte).is_err() {
            return Err(GitError::HeaderPackFileReadError);
        };
        println!("(MSB)Buffer: {:?}", byte);

        let seven_bits = (byte[0] & 0b01111111) as usize;
        // print_u8_bits(byte[0] & 0b01111111);
        println!(
            "(MSB)Unire:length seven: {:?} y length_bits: {}",
            seven_bits, length_bits
        );
        length_bits |= seven_bits << shift;
        println!("(MSB)Length final: {:?}", length_bits);
        if (byte[0] & 0x80) == 0 {
            break;
        }

        shift += 7;
    }
    Ok(length_bits)
}

fn create_object_bits(byte: u8) -> Result<ObjectType, GitError> {
    let byte = (byte & 0b01110000) >> 4;
    create_object(byte)
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
fn create_object(byte: u8) -> Result<ObjectType, GitError> {
    match byte {
        1 => Ok(ObjectType::Commit),
        2 => Ok(ObjectType::Tree),
        3 => Ok(ObjectType::Blob),
        4 => Ok(ObjectType::Tag),
        6 => Ok(ObjectType::OfsDelta),
        7 => Ok(ObjectType::RefDelta),
        _ => Err(GitError::InvalidObjectType),
    }
}

/// Lee desde el contenido descomprimido el tipo de objeto.
///
/// # Argumentos
///
/// * `decompressed_data`: El contenido de un objeto en bytes que puede ser blob, tree o coomit.
///
/// # Retorno
///
/// * `Ok(String::from_utf8_lossy(&type_object).to_string())`: Devuelve el tipo de objeto
/// * `Err(GitError)`: .
///
pub fn read_type(decompressed_data: &[u8]) -> Result<String, GitError> {
    let content = decompressed_data;
  
    let mut type_object: Vec<u8> = Vec::new();
    let mut index = 0;
    while index < content.len() && content[index] != SPACE {
        type_object.push(content[index]);
        index += 1;
    }

    Ok(String::from_utf8_lossy(&type_object).to_string())
}

/// Lee desde el contenido descomprimido el tipo de objeto.
///
/// # Argumentos
///
/// * `decompressed_data`: El contenido de un objeto en bytes que puede ser blob, tree o coomit.
///
/// # Retorno
///
/// * `Ok(String::from_utf8_lossy(&size).to_string())`: Devuelve el tamaño del objeto
/// * `Err(GitError)`: .
///
pub fn read_size(decompressed_data: &[u8]) -> Result<String, GitError> {
    let content = decompressed_data;
  
    let mut size: Vec<u8> = Vec::new();
    let mut index = 0;
    while index < content.len() && content[index] != SPACE {
        index += 1;
    }
    index += 1;
    while index < content.len() && content[index] != NULL {
        size.push(content[index]);
        index += 1;
    }
    Ok(String::from_utf8_lossy(&size).to_string())
}

/// Lee desde el contenido descomprimido el tipo de objeto de tipo tree.
///
/// # Argumentos
///
/// * `decompressed_data`: El contenido de un objeto en bytes de tipo tree.
///
/// # Retorno
///
/// * `Ok(String::from_utf8_lossy(&size).to_string())`: Devuelve el contenido del objeto (blobs o sub-tree) con el nombre
///     del archivo y su hash
/// * `Err(GitError)`: .
///
pub fn read_tree(decompressed_data: &[u8]) -> Result<String, GitError> {
    let content = decompressed_data;

    let mut index = 0;
    while index < content.len() && content[index] != NULL {
        index += 1;
    }
    index += 1;
    let mut result = String::new();

    while index < content.len() {
        let mut type_object: Vec<u8> = Vec::new();
        while index < content.len() && content[index] != SPACE {
            type_object.push(content[index]);
            index += 1;
        }
        let mut file_name: Vec<u8> = Vec::new();
        while index < content.len() && content[index] != NULL {
            file_name.push(content[index]);
            index += 1;
        }
        let mut hash: Vec<u8> = Vec::new();
        for _i in 0..20 {
            index += 1;
            hash.push(content[index]);
        }
        let hex_string = hash
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<String>();
        let object_format = format!(
            "{} {} {}\n",
            String::from_utf8_lossy(&type_object),
            String::from_utf8_lossy(&file_name),
            hex_string
        );
        result = result + &object_format;

        index += 1;
    }
    Ok(result)
}

/// Lee desde el contenido descomprimido el tipo de objeto de tipo commit.
///
/// # Argumentos
///
/// * `decompressed_data`: El contenido de un objeto en bytes de tipo commit.
///
/// # Retorno
///
/// * `Ok(String::from_utf8_lossy(&size).to_string())`: Devuelve el contenido del objeto
/// * `Err(GitError)`: .
///
pub fn read_commit(decompressed_data: &[u8]) -> Result<String, GitError> {
    let result_normal = decompressed_data;

    let mut index = 0;
    while index < result_normal.len() && result_normal[index] != NULL {
        index += 1;
    }

    index += 1;
    Ok(String::from_utf8_lossy(&decompressed_data[index..]).to_string())
}

/// Lee desde el contenido descomprimido el tipo de objeto de tipo blob.
///
/// # Argumentos
///
/// * `decompressed_data`: El contenido de un objeto en bytes de tipo blob.
///
/// # Retorno
///
/// * `Ok(String::from_utf8_lossy(&size).to_string())`: Devuelve el contenido del objeto
/// * `Err(GitError)`: .
///
pub fn read_blob(decompressed_data: &[u8]) -> Result<String, GitError> {
    let result_normal = decompressed_data;

    let mut index = 0;
    while index < result_normal.len() && result_normal[index] != NULL {
        index += 1;
    }
    index += 1;
    Ok(String::from_utf8_lossy(&decompressed_data[index..]).to_string())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

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

    #[test]
    fn test_read_size_encoded_length() {
        // Simulamos un Reader con datos de entrada
        let data: Vec<u8> = vec![
            0b10001010, // 138 en decimal
            0b01011000, // 88 en decimal
        ];

        // Creamos un cursor para simular la entrada de datos
        let mut cursor = Cursor::new(data);

        // Leemos la longitud codificada
        let result = read_size_encoded_length(&mut cursor, 0b10011010); // 154 en decimal
        assert_eq!(result, Ok(180394));
    }

    #[test]
    fn test_read_type_and_length() {
        // Simulamos un Reader con datos de entrada
        let data: Vec<u8> = vec![
            0b10011010, // 154 en decimal
            0b10001010, // 138 en decimal
            0b01011000, // 88 en decimal
        ];

        // Creamos un cursor para simular la entrada de datos
        let mut cursor = Cursor::new(data);

        let result = read_type_and_length(&mut cursor);
        assert_eq!(
            result,
            Ok(ObjectEntry {
                obj_type: ObjectType::Commit,
                obj_length: 180394
            })
        );
    }

    #[test]
    fn test_read_type_and_length_2() {
        // Simulamos un Reader con datos de entrada
        let data: Vec<u8> = vec![
            0b10011010, // 154 en decimal
            0b10001010, // 138 en decimal
            0b01011000, // 88 en decimal
            0b01001010, //  74 en decimal
            0b11101111, // 239 en decimal
            0b01011000, //  88 en decimal
        ];

        // Creamos un cursor para simular la entrada de datos
        let mut cursor = Cursor::new(data);

        let result = read_type_and_length(&mut cursor);
        assert_eq!(
            result,
            Ok(ObjectEntry {
                obj_type: ObjectType::Commit,
                obj_length: 180394
            })
        );

        let result = read_type_and_length(&mut cursor);
        assert_eq!(
            result,
            Ok(ObjectEntry {
                obj_type: ObjectType::Tag,
                obj_length: 10
            })
        );

        let result = read_type_and_length(&mut cursor);
        assert_eq!(
            result,
            Ok(ObjectEntry {
                obj_type: ObjectType::OfsDelta,
                obj_length: 1423
            })
        );
    }

    #[test]
    fn test_read_type() {
        let decompressed_data: Vec<u8> = vec![
            116, 114, 101, 101, 32, 54, 56, 0, 49, 48, 48, 54, 52, 52, 32, 67, 97, 114, 103, 111,
            46, 108, 111, 99, 107, 0, 231, 136, 208, 79, 233, 187, 237, 87, 61, 50, 240, 176, 36,
            147, 156, 178, 32, 76, 106, 134, 52, 48, 48, 48, 48, 32, 115, 114, 99, 0, 9, 50, 128,
            9, 9, 45, 28, 226, 40, 36, 92, 101, 118, 100, 240, 105, 241, 130, 145, 202,
        ];

        let type_object = read_type(&decompressed_data).expect("Error al leer el tipo");

        assert_eq!(type_object, "tree");
    }
    #[test]
    fn test_read_size() {
        let decompressed_data: Vec<u8> = vec![
            116, 114, 101, 101, 32, 54, 56, 0, 49, 48, 48, 54, 52, 52, 32, 67, 97, 114, 103, 111,
            46, 108, 111, 99, 107, 0, 231, 136, 208, 79, 233, 187, 237, 87, 61, 50, 240, 176, 36,
            147, 156, 178, 32, 76, 106, 134, 52, 48, 48, 48, 48, 32, 115, 114, 99, 0, 9, 50, 128,
            9, 9, 45, 28, 226, 40, 36, 92, 101, 118, 100, 240, 105, 241, 130, 145, 202,
        ];

        let type_object = read_size(&decompressed_data).expect("Error al leer el tipo");

        assert_eq!(type_object, "68");
    }
    #[test]
    fn test_read_tree() {
        let decompressed_data: Vec<u8> = vec![
            116, 114, 101, 101, 32, 54, 56, 0, 49, 48, 48, 54, 52, 52, 32, 67, 97, 114, 103, 111,
            46, 108, 111, 99, 107, 0, 231, 136, 208, 79, 233, 187, 237, 87, 61, 50, 240, 176, 36,
            147, 156, 178, 32, 76, 106, 134, 52, 48, 48, 48, 48, 32, 115, 114, 99, 0, 9, 50, 128,
            9, 9, 45, 28, 226, 40, 36, 92, 101, 118, 100, 240, 105, 241, 130, 145, 202,
        ];

        let tree = read_tree(&decompressed_data).expect("Error al leer el tipo");

        assert_eq!(tree, "100644  Cargo.lock e788d04fe9bbed573d32f0b024939cb2204c6a86\n40000  src 09328009092d1ce228245c657664f069f18291ca\n");
    }
    #[test]
    fn test_read_commit() {
        let decompressed_data: Vec<u8> = vec![
            99, 111, 109, 109, 105, 116, 32, 49, 57, 49, 0, 116, 114, 101, 101, 32, 55, 49, 50, 97,
            48, 55, 56, 97, 102, 48, 100, 50, 54, 97, 98, 48, 52, 50, 48, 101, 53, 48, 52, 55, 100,
            53, 99, 53, 101, 102, 50, 102, 56, 102, 57, 100, 102, 99, 100, 56, 10, 97, 117, 116,
            104, 111, 114, 32, 83, 50, 74, 117, 97, 110, 83, 50, 32, 60, 106, 117, 97, 110, 115,
            100, 101, 108, 114, 105, 111, 64, 104, 111, 116, 109, 97, 105, 108, 46, 99, 111, 109,
            62, 32, 49, 54, 57, 56, 53, 54, 49, 50, 52, 54, 32, 45, 48, 51, 48, 48, 10, 99, 111,
            109, 109, 105, 116, 116, 101, 114, 32, 83, 50, 74, 117, 97, 110, 83, 50, 32, 60, 106,
            117, 97, 110, 115, 100, 101, 108, 114, 105, 111, 64, 104, 111, 116, 109, 97, 105, 108,
            46, 99, 111, 109, 62, 32, 49, 54, 57, 56, 53, 54, 49, 50, 52, 54, 32, 45, 48, 51, 48,
            48, 10, 10, 112, 114, 117, 101, 98, 97, 32, 99, 111, 110, 32, 118, 97, 114, 105, 111,
            115, 32, 116, 114, 101, 101, 10,
        ];

        let commit = read_commit(&decompressed_data).expect("Error al leer el tipo");

        assert_eq!(commit, "tree 712a078af0d26ab0420e5047d5c5ef2f8f9dfcd8\nauthor S2JuanS2 <juansdelrio@hotmail.com> 1698561246 -0300\ncommitter S2JuanS2 <juansdelrio@hotmail.com> 1698561246 -0300\n\nprueba con varios tree\n");
    }
    #[test]
    fn test_read_blob() {
        let decompressed_data: Vec<u8> = vec![
            98, 108, 111, 98, 32, 49, 54, 0, 119, 104, 97, 116, 32, 105, 115, 32, 117, 112, 44, 32,
            100, 111, 99, 63,
        ];

        let blob = read_blob(&decompressed_data).expect("Error al leer el tipo");

        assert_eq!(blob, "what is up, doc?");
    }
}
