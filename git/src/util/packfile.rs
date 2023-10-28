use crate::{consts::PACK_SIGNATURE, errors::GitError, util::objects::read_type_and_length};
use std::{io::{Read, Write}, fs::File};
use flate2::{read::ZlibDecoder, write::ZlibEncoder};
use flate2::Compression;

use super::pkt_line::read;


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

pub fn read_packfile_header(reader: &mut dyn Read) -> Result<u32, GitError> {
    read_signature(reader)?;
    println!("Signature: {}", PACK_SIGNATURE);

    let version = read_version(reader)?;
    println!("Version: {}", version);

    let number_object = read_objects_contained(reader)?;
    println!("Number of objects: {}", number_object);
    Ok(number_object)
}

pub fn read_packfile_data(reader: &mut dyn Read, objects: usize) -> Result<(), GitError> {
    for _ in 0..objects {
        let mut buffer = [0u8; 132];
        reader.read_exact(&mut buffer).expect("Error al leer del servidor");
        let object_entry = read_type_and_length(reader)?;
        println!("Object entry: {:?}", object_entry);
        read_object_data(reader, object_entry.obj_length)?;
        let mut buffer = Vec::new();
        let a = reader.read_to_end(&mut buffer).expect("Error al leer del servidor");
        println!("Buffer: {:?}", buffer);
        println!("Len: {:?}", buffer.len());
        println!("a: {:?}", a);
    }
    Ok(())
}

fn read_object_data(reader: &mut dyn Read, object_length: usize) -> Result<u8, GitError> {
    // let mut count = 0;
    let mut buffer = vec![0; 250];
    reader.read(&mut buffer).expect("Error al leer del servidor");
    println!("Buffer: {:?}", buffer);

    let mut decompressed_data = Vec::new();

    let mut zlib_decoder: ZlibDecoder<&[u8]> = ZlibDecoder::new(&buffer[..]);
    // let mut zlib_decoder = ZlibDecoder::new(reader);
    let n = zlib_decoder.read_to_end(&mut decompressed_data).unwrap();
    println!("n: {}", n);
    println!("Len: {:?}", decompressed_data.len());
    println!("Descomprimido: {:?}", decompressed_data);
    let bytes_read = zlib_decoder.total_in();
    println!("Bytes read: {}", bytes_read);
    Ok(0)
}

// fn compress_data(data: &[u8]) -> Result<(), GitError> {
//     let compressed_data = Vec::new();
//     let mut encoder = ZlibEncoder::new(compressed_data, Compression::default());
//     match encoder.write_all(data)
//     {
//         Ok(_) => (),
//         Err(e) => {
//             println!("Error: {}", e);
//             return Err(GitError::GenericError);
//         }
//     };
//     let a = match encoder.finish()
//     {
//         Ok(a) => a,
//         Err(e) => {
//             println!("Error: {}", e);
//             return Err(GitError::GenericError);
//         }
//     };
//     println!("Compressed data: {:?}", a);
//     Ok(())
// }
// pub fn read_pack_prueba(reader: &mut dyn Read) -> Result<(), GitError>
// {
    // // Crear un buffer para almacenar los datos recibidos
    // let mut server_response = Vec::new();
    // let mut buffer = [0; 4096]; // Tamaño del buffer, ajusta según tus necesidades

    // // Leer los datos del servidor y almacenarlos en server_response
    // loop {
    //     let bytes_read = reader.read(&mut buffer).expect("Error al leer del servidor");
    //     if bytes_read == 0 {
    //         break; // No quedan más datos por leer
    //     }
    //     server_response.extend_from_slice(&buffer[..bytes_read]);
    // }

    // println!("Recibido: {:?}", server_response);
    // println!("Len: {:?}", server_response.len());
    // // Supongamos que `server_response` contiene los datos del packfile recibidos del servidor
    // let mut zlib_decoder = ZlibDecoder::new(&server_response[..]);
    // // Crear un vector para almacenar los datos descomprimidos
    // let mut decompressed_data = Vec::new();
    // zlib_decoder.read_to_end(&mut decompressed_data).expect("Error al descomprimir el packfile");
    // println!("Descomprimido: {:?}", decompressed_data);
    // Ok(())
    // Crea un archivo .pack en el sistema de archivos local
//         let mut file = match File::create("nombre_del_archivo.pack")
//         {
//             Ok(file) => file,
//             Err(e) => {
//                 println!("Error: {}", e);
//                 return Err(GitError::GenericError);
//             }
//         };

//         // Lee del stream y copia los datos al archivo
//         let mut buffer = [0u8; 4096]; // Puedes ajustar el tamaño del búfer según tus necesidades
//         loop {
//             let bytes_read = match reader.read(&mut buffer)
//             {
//                 Ok(bytes_read) => bytes_read,
//                 Err(e) => {
//                     println!("Error: {}", e);
//                     return Err(GitError::GenericError);
//                 }
//             };
//             if bytes_read == 0 {
//                 break; // Se llegó al final del archivo
//             }
//             match file.write_all(&buffer[..bytes_read])
//             {
//                 Ok(_) => (),
//                 Err(e) => {
//                     println!("Error: {}", e);
//                     return Err(GitError::GenericError);
//                 }
//             };
//             file.flush().expect("Error al escribir en el archivo");
//         }

//         let lala = match reader.read(&mut buffer)
//         {
//             Ok(bytes_read) => bytes_read,
//             Err(e) => {
//                 println!("Error: {}", e);
//                 return Err(GitError::GenericError);
//             }
//         };
//         println!("Lala: {:?}", lala);
//         println!("HOla");
//         Ok(())
// }

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
