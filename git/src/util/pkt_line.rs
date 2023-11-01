use std::io::Read;

use crate::{consts::LENGTH_PREFIX_SIZE, errors::GitError};

/// Lee líneas de paquete del flujo de entrada proporcionado y las devuelve como un vector de vectores de bytes.
///
/// Esta función continúa leyendo líneas de paquete del flujo hasta encontrar una línea de paquete vacía,
/// que indica el final del paquete.
///
/// Cada línea de paquete se lee utilizando la función `read_pkt_line`, que espera que el flujo de entrada
/// contenga líneas de paquete según el formato de Git.
///
/// Si ocurre un error al leer cualquiera de las líneas de paquete, se devuelve un error `GitError`.
///
/// # Argumentos
///
/// - `stream`: Un mutable referencia a un objeto que implementa el trait `Read` para la entrada de datos.
///
/// # Retorno
///
/// - `Result<Vec<Vec<u8>>, GitError>`: Un resultado que contiene un vector de vectores de bytes,
///   donde cada vector representa una línea de paquete del paquete Git leído.
///   Si ocurre un error, se devuelve el error correspondiente.
pub fn read(stream: &mut dyn Read) -> Result<Vec<Vec<u8>>, GitError> {
    let mut lines: Vec<Vec<u8>> = Vec::new();

    loop {
        match read_pkt_line(stream) {
            Ok(line) => {
                if line.is_empty() {
                    return Ok(lines);
                }
                lines.push(line);
            }
            Err(err) => return Err(err),
        }
    }
}

/// Lee una línea de paquete del flujo de entrada proporcionado y la devuelve como un vector de bytes.
///
/// Esta función espera que el flujo de entrada contenga líneas de paquete según el formato de Git:
///
/// 1. Lea los primeros 4 bytes que representan la longitud de la línea en formato hexadecimal.
/// 2. Convierte la longitud hexadecimal a un número entero.
/// 3. Lee la cantidad de bytes especificada por la longitud menos 1 (debido al carácter de nueva línea).
/// 4. Devuelve el contenido de la línea de paquete como un vector de bytes.
///
/// Si ocurre un error al leer la línea de paquete o al convertir la longitud hexadecimal,
/// se devuelve un error `GitError::InvalidPacketLineError`.
///
/// Si la longitud es 0, lo que indica el final del paquete, se devuelve un vector de bytes vacío.
///
/// # Argumentos
///
/// - `socket`: Un mutable referencia a un objeto que implementa el trait `Read` para la entrada de datos.
///
/// # Retorno
///
/// - `Result<Vec<u8>, GitError>`: Un resultado que contiene el contenido de la línea de paquete o un error si ocurre alguno.
fn read_pkt_line(socket: &mut dyn Read) -> Result<Vec<u8>, GitError> {
    let mut length_buf = [0u8; 4];
    if socket.read_exact(&mut length_buf).is_err() {
        return Err(GitError::InvalidPacketLineError);
    };

    let length_hex = String::from_utf8_lossy(&length_buf);
    let length = match u32::from_str_radix(length_hex.trim(), 16) {
        Ok(l) => l,
        Err(_) => return Err(GitError::InvalidPacketLineError),
    };

    if length == 0 {
        // End of the packet
        return Ok(vec![]);
    }

    let length = length as usize - LENGTH_PREFIX_SIZE - 1; // 1 por el enter
    let mut content = vec![0u8; length];
    if socket.read_exact(&mut content).is_err() {
        return Err(GitError::InvalidPacketLineError);
    };

    // Consume the newline character
    let mut newline_buf = [0u8; 1];
    if socket.read_exact(&mut newline_buf).is_err() {
        return Err(GitError::InvalidPacketLineError);
    };

    Ok(content)
}

pub fn read_line_from_bytes(bytes: &[u8]) -> Result<&[u8], GitError> {
    if bytes.len() < 4 {
        return Err(GitError::InvalidPacketLineError);
    }
    let len = match u32::from_str_radix(String::from_utf8_lossy(&bytes[0..4]).trim(), 16)
    {
        Ok(l) => l as usize,
        Err(_) => return Err(GitError::InvalidPacketLineError),
    };

    let data: &[u8] = &bytes[4..len];
    // let data

    Ok(data)
}

/// Agrega un prefijo de longitud a un mensaje y lo devuelve como una cadena.
///
/// Esta función toma un mensaje y la longitud del mensaje sin el prefijo y agrega un
/// prefijo hexadecimal de 4 caracteres que indica la longitud del mensaje completo.
///
/// # Argumentos
///
/// * `message`: Un mensaje al que se le agregará el prefijo de longitud.
/// * `len`: La longitud del mensaje original (sin el prefijo), el motivo de este argumento
///     es porque algunas cadenas contienen \0 y la función `len()` no funciona correctamente.
///
/// # Ejemplo
///
/// ```
/// use git::util::pkt_line::add_length_prefix;
///
/// let message = "Hola, mundo!";
/// let length = message.len();
///
/// let prefixed_message = add_length_prefix(message, length);
///
/// // El resultado será una cadena con el prefijo de longitud:
/// // "0010Hola, mundo!"
/// ```
///
/// # Retorno
///
/// Devuelve una cadena que contiene el mensaje original con un prefijo de longitud hexadecimal.
///
/// Nota: El prefijo de longitud se calcula automáticamente y tiene una longitud fija de 4 caracteres.
///
pub fn add_length_prefix(message: &str, len: usize) -> String {
    // Obtener la longitud del mensaje con el prefijo
    let message_length = len + LENGTH_PREFIX_SIZE;

    // Convertir la longitud en una cadena hexadecimal de 4 caracteres
    let length_hex = format!("{:04x}", message_length);

    // Concatenar la longitud al principio del mensaje
    let prefixed_message = format!("{}{}", length_hex, message);

    prefixed_message
}

#[cfg(test)]
mod tests {
    use crate::consts::FLUSH_PKT;

    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_pkt_line_valid() {
        // Crear un cursor con una cadena de longitud válida
        let input = "0012hello, world!\n";
        let mut cursor = Cursor::new(input);

        let result = read_pkt_line(&mut cursor);
        assert!(result.is_ok());

        let content = result.unwrap();
        assert_eq!(content, b"hello, world!");
    }

    #[test]
    fn test_read_pkt_line_empty() {
        // Crear un cursor con una cadena vacía (fin de paquete)
        let input = FLUSH_PKT.to_string();
        let mut cursor = Cursor::new(input);

        let result = read_pkt_line(&mut cursor);
        assert!(result.is_ok());

        let content = result.unwrap();
        assert_eq!(content, &[]);
    }

    #[test]
    fn test_read_pkt_line_invalid_length() {
        // Crear un cursor con una longitud no válida
        let input = "0zinvalid_length\n";
        let mut cursor = Cursor::new(input);

        let result = read_pkt_line(&mut cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_empty_stream() {
        let input = FLUSH_PKT.as_bytes();
        let mut stream = Cursor::new(input);

        let result = read(&mut stream);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Vec::<Vec<u8>>::new());
    }

    #[test]
    fn test_read_single_line() {
        let input = format!("0012Hello, world!\n{}", FLUSH_PKT);
        let mut stream = Cursor::new(input.as_bytes());

        let result = read(&mut stream);
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], b"Hello, world!");
    }

    #[test]
    fn test_read_multiple_lines() {
        let input = format!("000bLine 1\n000bLine 2\n000bLine 3\n{}", FLUSH_PKT);
        let mut stream = Cursor::new(input.as_bytes());

        let result = read(&mut stream);

        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], b"Line 1");
        assert_eq!(lines[1], b"Line 2");
        assert_eq!(lines[2], b"Line 3");
    }

    #[test]
    fn test_read_incomplete_line() {
        let input = b"0036Incomplete Line";
        let mut stream = Cursor::new(input);

        let result = read(&mut stream);

        assert!(result.is_err());
    }

    #[test]
    fn test_add_length_prefix() {
        let message = "7217a7c7e582c46cec22a130adf4b9d7d950fba0 HEAD\0multi_ack thin-pack side-band side-band-64k ofs-delta shallow no-progress include-tag\n";

        let prefixed_message = add_length_prefix(message, 132);

        assert_eq!(prefixed_message, "00887217a7c7e582c46cec22a130adf4b9d7d950fba0 HEAD\0multi_ack thin-pack side-band side-band-64k ofs-delta shallow no-progress include-tag\n");
    }

    #[test]
    fn test_add_length_prefix_incorrect() {
        let message = "Hola, mundo!";
        let length = message.len();

        // Le quito 1 al len para que me de un prefijo incorrecto
        let prefixed_message = add_length_prefix(message, length - 1);

        // assert_eq!(prefixed_message, incorrect_reference);
        assert_ne!(prefixed_message, "0010Hola, mundo!");
    }
}
