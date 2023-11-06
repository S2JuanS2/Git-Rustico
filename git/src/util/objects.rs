use crate::consts::*;
use crate::errors::GitError;
use crate::util::files::create_directory;
use crate::util::formats::{compressor_object, hash_generate};
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::errors::UtilError;
use super::formats::{compressor_object_with_bytes, hash_generate_with_bytes};

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

/// Convierte un objeto en bytes, siguiendo las reglas de la especificación Git Pack.
///
/// # Ejemplo
///
/// ```rust
/// use crate::util::objects::{ObjectEntry, ObjectType};
///
/// let object = ObjectType::new(ObjectType::Blob, 50);
/// let bytes = object.to_bytes();
/// ```
/// # Argumentos
/// * `obj_type` - Tipo de objeto.
/// * `obj_length` - Longitud del objeto.
///
/// # Retorno
/// * `Vec<u8>` - Vector de bytes que representa el objeto.
///
impl ObjectEntry {
    pub fn new(obj_type: ObjectType, obj_length: usize) -> ObjectEntry {
        ObjectEntry {
            obj_type,
            obj_length,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        let mut byte: u8 = 0;
        match &self.obj_type {
            ObjectType::Commit => byte |= 0b00010000,
            ObjectType::Tree => byte |= 0b00100000,
            ObjectType::Blob => byte |= 0b00110000,
            ObjectType::Tag => byte |= 0b01000000,
            ObjectType::OfsDelta => byte |= 0b01100000,
            ObjectType::RefDelta => byte |= 0b01110000,
        };

        if self.obj_length < 15 {
            byte |= self.obj_length as u8;
            bytes.push(byte);
            return bytes;
        }
        let rest = (self.obj_length & 0b00001111) as u8;
        byte |= rest;
        byte |= 0b10000000;
        bytes.push(byte);
        encode_size_encoding(self.obj_length, 4, &mut bytes);
        bytes
        // return
    }
}

/// Codifica un número usando el esquema de codificación de tamaño definido por Git Pack.
///
/// # Argumentos
///
/// * `number` - Número a codificar.
/// * `offset` - Desplazamiento para el número.
/// * `result` - Vector que almacena los bytes codificados.
///
pub fn encode_size_encoding(mut number: usize, offset: u8, result: &mut Vec<u8>) {
    number >>= offset;
    loop {
        let mut byte = (number & 0b01111111) as u8;
        number >>= 7;
        if number != 0 {
            byte |= 0b10000000;
        }
        result.push(byte);
        if number == 0 {
            break;
        }
    }
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
    let mut buffer: [u8; 1] = [0u8; 1];
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
) -> Result<ObjectEntry, UtilError> {
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
) -> Result<usize, UtilError> {
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

fn create_object_bits(byte: u8) -> Result<ObjectType, UtilError> {
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
fn create_object(byte: u8) -> Result<ObjectType, UtilError> {
    match byte {
        1 => Ok(ObjectType::Commit),
        2 => Ok(ObjectType::Tree),
        3 => Ok(ObjectType::Blob),
        4 => Ok(ObjectType::Tag),
        6 => Ok(ObjectType::OfsDelta),
        7 => Ok(ObjectType::RefDelta),
        _ => Err(UtilError::InvalidObjectType),
    }
}

/// Creará la carpeta con los 2 primeros digitos del hash del objeto commit, y el archivo con los ultimos 38 de nombre.
fn builder_object(git_dir: &str, hash_object: &str) -> Result<File, GitError> {
    let objects_dir = format!(
        "{}/{}/{}/{}",
        &git_dir,
        DIR_OBJECTS,
        &hash_object[..2],
        &hash_object[2..]
    );

    let hash_object_path = format!("{}/{}/{}/", &git_dir, DIR_OBJECTS, &hash_object[..2]);

    create_directory(Path::new(&hash_object_path))?;

    let file_object = match File::create(objects_dir) {
        Ok(file_object) => file_object,
        Err(_) => return Err(GitError::CreateFileError),
    };

    Ok(file_object)
}

/// comprimirá el contenido y lo escribirá en el archivo
/// ###Parametros:
/// 'git_dir': Directorio del git
/// 'content': contenido del archivo a comprimir
pub fn builder_object_blob(content: Vec<u8>, git_dir: &str) -> Result<String, GitError> {
    let header = format!("{} {}\0", BLOB, content.len());
    let store = header + String::from_utf8_lossy(&content).as_ref();

    let hash_blob = hash_generate(&store);

    let file_object = builder_object(git_dir, &hash_blob)?;

    compressor_object(store, file_object)?;

    Ok(hash_blob)
}

/// comprimirá el contenido y lo escribirá en el archivo
/// ###Parametros:
/// 'git_dir': Directorio del git
/// 'hash_commit': hash del objeto commit previamente generado
pub fn builder_object_commit(content: &str, git_dir: &str) -> Result<String, GitError> {
    let content_bytes = content.as_bytes();
    let content_size = content_bytes.len().to_string();
    let header = format!("commit {}\0", content_size);

    let store = header + content;

    let hash_commit = hash_generate(&store);

    let file = builder_object(git_dir, &hash_commit)?;
    compressor_object(store, file)?;

    Ok(hash_commit)
}

fn read_index_clone(content: &str) -> Result<Vec<u8>, GitError> {
    let mut format_tree = Vec::new();

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let file_name = parts[0];
        let mut mode = parts[1];
        let hash = parts[2];

        if mode == BLOB {
            mode = FILE;
        } else if mode == TREE {
            mode = DIRECTORY;
        }
        let bytes = hash
            .as_bytes()
            .chunks(2)
            .filter_map(|chunk| {
                let hex_str = String::from_utf8_lossy(chunk);
                u8::from_str_radix(&hex_str, 16).ok()
            })
            .collect::<Vec<u8>>();

        format_tree.extend_from_slice(file_name.as_bytes());
        format_tree.push(SPACE);
        format_tree.extend_from_slice(mode.as_bytes());
        format_tree.push(NULL);
        format_tree.extend_from_slice(&bytes);
    }
    Ok(format_tree)
}

pub fn builder_object_tree_clone(git_dir: &str, content: &str) -> Result<String, GitError> {
    println!("content: {}", content);
    let format_tree = read_index_clone(content)?;

    let content_size = format_tree.len().to_string();
    println!("{}", content_size);
    let tree_format = "tree ";
    let mut header: Vec<u8> = vec![];
    header.extend_from_slice(tree_format.as_bytes());
    header.extend_from_slice(&content_size.as_bytes());
    header.push(NULL);
    header.extend_from_slice(&format_tree);
    let hash_tree = hash_generate_with_bytes(header.clone());

    let file = builder_object(git_dir, &hash_tree)?;

    compressor_object_with_bytes(header, file)?;

    Ok(hash_tree)
}

fn read_index(git_dir: &str) -> Result<Vec<u8>, GitError> {
    let path_index = format!("{}/{}", git_dir, INDEX);

    let content_bytes = match fs::read(path_index) {
        Ok(content_bytes) => content_bytes,
        Err(_) => return Err(GitError::OpenFileError),
    };
    let mut format_tree = Vec::new();
    let content_index = String::from_utf8_lossy(&content_bytes);

    for line in content_index.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let file_name = parts[0];
        let mut mode = parts[1];
        let hash = parts[2];

        if mode == BLOB {
            mode = FILE;
        } else if mode == TREE {
            mode = DIRECTORY;
        }
        let bytes = hash
            .as_bytes()
            .chunks(2)
            .filter_map(|chunk| {
                let hex_str = String::from_utf8_lossy(chunk);
                u8::from_str_radix(&hex_str, 16).ok()
            })
            .collect::<Vec<u8>>();

        format_tree.extend_from_slice(file_name.as_bytes());
        format_tree.push(SPACE);
        format_tree.extend_from_slice(mode.as_bytes());
        format_tree.push(NULL);
        format_tree.extend_from_slice(&bytes);
    }
    Ok(format_tree)
}

pub fn builder_object_tree(git_dir: &str) -> Result<String, GitError> {
    let format_tree = read_index(git_dir)?;

    let content_size = format_tree.len().to_string();
    println!("{}", content_size);
    let tree_format = "tree ";
    let mut header: Vec<u8> = vec![];
    header.extend_from_slice(tree_format.as_bytes());
    header.extend_from_slice(&content_size.as_bytes());
    header.push(NULL);
    header.extend_from_slice(&format_tree);
    let hash_tree = hash_generate_with_bytes(header.clone());

    let file = builder_object(git_dir, &hash_tree)?;

    compressor_object_with_bytes(header, file)?;

    Ok(hash_tree)
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

pub fn read_tree_content(decompressed_data: &[u8]) -> Result<String, GitError> {
    let content = decompressed_data;

    let mut index = 0;
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
        index += 1;
        let mut hash: Vec<u8> = Vec::new();
        for _i in 0..20 {
            if index < content.len() {
                hash.push(content[index]);
                index += 1;
            }
        }
        index -= 1;
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
        index += 1;
        let mut hash: Vec<u8> = Vec::new();
        for _i in 0..20 {
            if index < content.len() {
                hash.push(content[index]);
                index += 1;
            }
        }
        index -= 1;
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

pub fn read_commit_content(decompressed_data: &[u8]) -> Result<String, GitError> {
    Ok(String::from_utf8_lossy(decompressed_data).to_string())
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

pub fn read_blob_content(decompressed_data: &[u8]) -> Result<String, GitError> {
    Ok(String::from_utf8_lossy(decompressed_data).to_string())
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
        assert_eq!(object_type, Err(UtilError::InvalidObjectType));
    }

    #[test]
    fn test_create_object_invalid_type_0() {
        let object_type = create_object(0); // Tipo inválido
        assert_eq!(object_type, Err(UtilError::InvalidObjectType));
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

    #[test]
    fn test_encode_size_encoding_100() {
        let mut bytes: Vec<u8> = Vec::new();
        encode_size_encoding(100, 0, &mut bytes);
        assert_eq!(bytes, vec![0b01100100]);
    }

    #[test]
    fn test_encode_size_encoding_200() {
        let mut bytes: Vec<u8> = Vec::new();
        encode_size_encoding(200, 0, &mut bytes);
        assert_eq!(bytes, vec![0b11001000, 0b00000001]);
    }

    #[test]
    fn test_encode_size_encoding_128() {
        let mut bytes: Vec<u8> = Vec::new();
        encode_size_encoding(128, 0, &mut bytes);
        assert_eq!(bytes, vec![0b10000000, 0b00000001]);
    }

    #[test]
    fn test_encode_size_encoding_300() {
        let mut bytes: Vec<u8> = Vec::new();
        encode_size_encoding(300, 0, &mut bytes);
        assert_eq!(bytes, vec![0b10101100, 0b00000010]);
    }

    #[test]
    fn test_encode_size_encoding_99999() {
        let mut bytes: Vec<u8> = Vec::new();
        encode_size_encoding(99999, 0, &mut bytes);
        assert_eq!(bytes, vec![0b10011111, 0b10001101, 0b00000110]);
    }

    #[test]
    fn test_to_bytes_with_small_obj_length() {
        let object = ObjectEntry::new(ObjectType::Blob, 10);
        let bytes = object.to_bytes();

        assert_eq!(bytes.len(), 1);
        assert_eq!(bytes[0], 0b00111010); // Assuming Blob type for 10-byte length
    }

    #[test]
    fn test_to_bytes_with_large_obj_length() {
        let object = ObjectEntry::new(ObjectType::Tree, 300);
        let bytes = object.to_bytes();
        println!("Bytes: {:?}", bytes);
        assert_eq!(bytes.len(), 2);
        assert_eq!(bytes[0], 0b10101100);
        assert_eq!(bytes[1], 0b00010010);
    }
}
