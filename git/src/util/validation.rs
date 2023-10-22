use std::path::Path;

use crate::{consts::*, errors::GitError};

/// Valida una dirección IP.
///
/// Esta función toma una cadena `input` que representa una dirección IP y verifica si es una
/// dirección IP válida. Puede ser una dirección IPv4 o IPv6.
///
/// # Argumentos
///
/// * `input`: Una cadena que representa una dirección IP que se desea validar.
///
/// # Ejemplo
///
/// ```
/// use git::util::validation::valid_ip;
/// use git::errors::GitError;
///
/// match valid_ip("192.168.1.1") {
///     Ok(ip) => println!("La dirección IP {} es válida.", ip),
///     Err(GitError) => println!("La dirección IP no es válida."),
/// }
/// ```
///
/// # Retorno
///
/// * `Ok(input)`: Si la dirección IP es válida (IPv4 o IPv6).
/// * `Err(GitError::InvalidIpError)`: Si la dirección IP no es válida.
///
/// Esta función verifica si `input` es una dirección IPv4 válida utilizando la función
/// `is_valid_ipv4` y si es una dirección IPv6 válida utilizando la función `is_ipv6`. Si
/// alguna de las dos validaciones tiene éxito, se considera que la dirección IP es válida.
///
/// # Errores
///
/// Además del `GitError::InvalidIpError`, esta función puede devolver otros errores si
/// ocurren durante la validación.
///
pub fn valid_ip(input: &str) -> Result<String, GitError> {
    if is_valid_ipv4(input) || is_ipv6(input) {
        return Ok(input.to_string());
    }
    Err(GitError::InvalidIpError)
}

/// Valida un número de puerto.
///
/// Esta función verifica si una cadena `input` es un número de puerto válido. Para que un número
/// sea considerado válido, debe cumplir con las siguientes condiciones:
///
/// 1. Debe ser un número entero positivo.
/// 2. Debe estar dentro del rango permitido de puertos, definido por las constantes `PORT_MIN` y
///    `PORT_MAX`.
///
/// # Argumentos
///
/// * `input`: Una cadena que representa el número de puerto que se desea validar.
///
/// # Ejemplo
///
/// ```
/// use git::util::validation::valid_port;
/// use git::errors::GitError;
///
/// match valid_port("8080") {
///     Ok(port) => println!("El número de puerto es válido: {}", port),
///     Err(GitError::InvalidPortError) => println!("El número de puerto no es válido."),
///     Err(_) => println!("Otro error."),
/// }
/// ```
///
/// # Retorno
///
/// * `Ok(input.to_string())`: Si el número de puerto es válido.
/// * `Err(GitError::InvalidPortError)`: Si el número de puerto no es válido.
///
/// Esta función realiza la validación verificando si el `input` es un número entero y si se encuentra
/// dentro del rango permitido. Si cumple con ambas condiciones, se considera válido y se devuelve
/// la cadena original en formato de texto. En caso de no cumplir con las condiciones, se devuelve
/// un error.
///
/// # Errores
///
/// Su unico error es `GitError::InvalidPortError`
///
pub fn valid_port(input: &str) -> Result<String, GitError> {
    if let Ok(num) = input.parse::<u16>() {
        if (num < PORT_MAX) & (num > PORT_MIN) {
            return Ok(input.to_string());
        }
    }
    Err(GitError::InvalidPortError)
}

/// Valida un path de archivo.
///
/// Esta función toma una cadena `input` que representa una ruta de archivo y valida si el
/// directorio padre de la ruta existe y es un directorio válido. Se utiliza comúnmente para
/// validar el path de un archivo de registro.
///
/// # Argumentos
///
/// * `input`: Una cadena que representa la ruta de archivo que se desea validar.
///
/// # Ejemplo
///
/// ```
/// use git::util::validation::valid_path_log;
/// use git::errors::GitError;
///
/// match valid_path_log("/var/log/myapp.log") {
///     Ok(path) => println!("El path del archivo de registro es válido: {}", path),
///     Err(e) => println!("{}", e.message()),
/// }
/// ```
///
/// # Retorno
///
/// * `Ok(input.to_string())`: Si el directorio padre del archivo es válido.
/// * `Err(GitError::InvalidLogDirectoryError)`: Si el directorio padre del archivo no es válido.
///
/// Esta función utiliza la biblioteca estándar de Rust (`std::path`) para obtener el directorio
/// padre del path de archivo proporcionado. Luego, verifica si ese directorio existe y si es un
/// directorio válido. Si el directorio es válido, devuelve el path de archivo original.
///
/// # Errores
///
/// Solo devuelve `GitError::InvalidLogDirectoryError`.
///
pub fn valid_path_log(input: &str) -> Result<String, GitError> {
    // Obtener el directorio padre del path del archivo
    if let Some(parent_dir) = Path::new(input).parent() {
        if parent_dir.exists() && parent_dir.is_dir() {
            Ok(input.to_string())
        } else {
            Err(GitError::InvalidLogDirectoryError)
        }
    } else {
        Err(GitError::InvalidLogDirectoryError)
    }
}

/// Valida una dirección de correo electrónico.
///
/// Esta función toma una cadena `input` que representa una dirección de correo electrónico y
/// verifica si cumple con ciertas condiciones para considerarse válida. Las direcciones de correo
/// electrónico válidas deben cumplir con las siguientes reglas:
///
/// 1. Deben contener exactamente un carácter '@' que divida la dirección en dos partes.
/// 2. El "local part" (la parte antes de '@') no debe estar vacío y no debe contener caracteres
///    inválidos.
/// 3. El "dominio" (la parte después de '@') debe contener al menos un punto ('.') y no debe
///    contener caracteres inválidos.
///
/// # Argumentos
///
/// * `input`: Una cadena que representa la dirección de correo electrónico que se desea validar.
///
/// # Ejemplo
///
/// ```
/// use git::util::validation::valid_email;
/// use git::errors::GitError;
///
/// match valid_email("usuario@ejemplo.com") {
///     Ok(email) => println!("La dirección de correo electrónico es válida: {}", email),
///     Err(e) => println!("{}", e.message()),
/// }
/// ```
///
/// # Retorno
///
/// * `Ok(input.to_string())`: Si la dirección de correo electrónico es válida.
/// * `Err(GitError::InvalidUserMailError)`: Si la dirección de correo electrónico no es válida.
///
/// Esta función verifica la validez de la dirección de correo electrónico realizando múltiples
/// comprobaciones. Si la dirección no cumple con las reglas especificadas, se devuelve un error.
///
/// # Errores
///
/// Solo devuelve `GitError::InvalidUserMailError`.
///
pub fn valid_email(input: &str) -> Result<String, GitError> {
    let parts: Vec<&str> = input.split('@').collect();

    if parts.len() != 2 {
        return Err(GitError::InvalidUserMailError);
    }

    let local_part = parts[0];
    let domain_part = parts[1];

    if !is_valid_local_part(local_part) {
        return Err(GitError::InvalidUserMailError);
    }

    if !is_valid_domain_part(domain_part) {
        return Err(GitError::InvalidUserMailError);
    }

    Ok(input.to_string())
}

/// Verifica si un objeto ID (obj_id) dado es válido.
///
/// Un objeto ID válido debe cumplir con los siguientes criterios:
///
/// 1. Debe tener una longitud de 40 caracteres, que es la longitud típica de un objeto ID en Git.
/// 2. Debe contener caracteres válidos que son dígitos hexadecimales (0-9, a-f, A-F).
///
/// # Argumentos
///
/// * `obj_id`: Un objeto ID (hash) que se desea validar.
///
/// # Ejemplo
///
/// ```
/// use git::util::validation::is_valid_obj_id;
/// let obj_id_valido = "7217a7c7e582c46cec22a130adf4b9d7d950fba0";
/// let obj_id_invalido = "invalid_hash";
///
/// assert_eq!(is_valid_obj_id(obj_id_valido), true);
/// assert_eq!(is_valid_obj_id(obj_id_invalido), false);
/// ```
///
/// # Retorno
///
/// `true` si el objeto ID es válido, `false` en caso contrario.
pub fn is_valid_obj_id(obj_id: &str) -> bool {
    if obj_id.len() != 40 {
        return false;
    }

    for c in obj_id.chars() {
        if !c.is_ascii_hexdigit() {
            return false;
        }
    }

    true
}

/// Comprueba si la cadena de entrada es una dirección IPv4 válida.
///
/// La función verifica si la cadena de entrada contiene cuatro segmentos separados
/// por puntos (octetos) y si cada segmento es un número decimal válido en el rango
/// de 0 a 255.
///
/// # Argumentos
///
/// * `input`: Una cadena de texto que se va a verificar como una dirección IPv4.
///
/// # Retorno
///
/// `true` si la cadena es una dirección IPv4 válida, de lo contrario, `false`.
fn is_valid_ipv4(input: &str) -> bool {
    let octets: Vec<&str> = input.split('.').collect();

    if octets.len() != IPV4_SECTIONS {
        return false;
    }

    for octet in octets {
        if let Ok(num) = octet.parse::<u16>() {
            if num > IPV4_MAX {
                return false;
            }
        } else {
            return false;
        }
    }

    true
}

/// Verifica si una cadena dada representa una dirección IPv6 válida.
///
/// Esta función comprueba si la cadena cumple con el formato de una dirección IPv6 válida,
/// que consiste en 8 secciones de 4 dígitos hexadecimales separados por dos puntos.
///
/// # Argumentos
///
/// * `ip`: Una cadena que se va a verificar como dirección IPv6.
///
/// # Retorno
///
/// Devuelve `true` si la cadena es una dirección IPv6 válida, y `false` en caso contrario.
fn is_ipv6(ip: &str) -> bool {
    let parts: Vec<&str> = ip.split(':').collect();

    if parts.len() != IPV6_SECTIONS {
        return false;
    }

    for part in parts {
        if part.len() != IPV4_SECTION_LENGTH && !(part.len() == 1 && part == "0") {
            return false;
        }

        if u32::from_str_radix(part, 16).is_err() {
            return false;
        }

        let value = match u32::from_str_radix(part, 16) {
            Ok(n) => n,
            Err(_) => return false,
        };

        if value > u32::from(IPV6_MAX) {
            return false;
        }
    }

    true
}

/// Verifica si la parte local de una dirección de correo electrónico es válida.
///
/// La parte local de una dirección de correo electrónico debe cumplir los siguientes criterios:
///
/// - No debe estar vacía.
/// - Debe consistir en caracteres alfanuméricos, puntos (.), guiones (-) o guiones bajos (_).
///
/// # Argumentos
///
/// - `local_part`: Una cadena que representa la parte local de una dirección de correo electrónico.
///
/// # Retorno
///
/// Retorna `true` si la parte local es válida, de lo contrario, retorna `false`.
fn is_valid_local_part(local_part: &str) -> bool {
    if local_part.is_empty() {
        return false;
    }
    for c in local_part.chars() {
        if !c.is_alphanumeric() && c != '.' && c != '-' && c != '_' {
            return false;
        }
    }
    true
}

/// Verifica si la parte de dominio de una dirección de correo electrónico es válida.
///
/// Para que la parte de dominio sea válida, debe cumplir con los siguientes criterios:
/// - No debe estar vacía.
/// - Debe contener al menos un punto (`.`) para separar subdominios.
/// - Cada subdominio debe contener solo caracteres alfanuméricos y guiones (`-`).
///
/// # Argumentos
/// - `domain_part`: Una cadena que representa la parte de dominio de una dirección de correo.
///
/// # Retorno
/// Un valor booleano (`true` si es válido, `false` si no lo es).
fn is_valid_domain_part(domain_part: &str) -> bool {
    if domain_part.is_empty() {
        return false;
    }

    let domain_parts: Vec<&str> = domain_part.split('.').collect();
    if domain_parts.len() < 2 {
        return false;
    }

    for part in domain_parts {
        for c in part.chars() {
            if !c.is_alphanumeric() && c != '-' {
                return false;
            }
        }
    }
    true
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_ip_v4() {
        let valid_ipv4 = "192.168.1.1";
        let result = valid_ip(valid_ipv4);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), valid_ipv4);
    }

    #[test]
    fn test_valid_ip_v6() {
        let valid_ipv6 = "2001:0db8:85a3:0000:0000:8a2e:0370:7334";
        let result = valid_ip(valid_ipv6);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), valid_ipv6);
    }

    #[test]
    fn test_invalid_ip() {
        let invalid_ip = "invalid";
        let result = valid_ip(invalid_ip);
        assert!(result.is_err());
        assert_eq!(result.err(), Some(GitError::InvalidIpError));
    }

    #[test]
    fn test_valid_port() {
        let port = "8080";
        let result = valid_port(port);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), port);
    }

    #[test]
    fn test_invalid_port_minimum_range() {
        let invalid_port_low = "10";
        let result = valid_port(invalid_port_low);
        assert!(result.is_err());
        assert_eq!(result.err(), Some(GitError::InvalidPortError));
    }

    #[test]
    fn test_invalid_port_maximum_range() {
        let invalid_port_high = "65536";
        let result = valid_port(invalid_port_high);
        assert!(result.is_err());
        assert_eq!(result.err(), Some(GitError::InvalidPortError));
    }

    #[test]
    fn test_invalid_port_non_numeric() {
        let invalid_port_non_numeric = "abc";
        let result = valid_port(invalid_port_non_numeric);
        assert!(result.is_err());
        assert_eq!(result.err(), Some(GitError::InvalidPortError));
    }

    #[test]
    fn test_valid_path_log() {
        let valid_path = "./archivo.log";
        let result = valid_path_log(valid_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), valid_path);
    }

    #[test]
    fn test_invalid_path_log_non_existent_directory() {
        let invalid_path_nonexistent = "./no_existe/archivo_inexistente.txt";
        let result = valid_path_log(invalid_path_nonexistent);
        assert!(result.is_err());
        assert_eq!(result.err(), Some(GitError::InvalidLogDirectoryError));
    }

    #[test]
    fn test_invalid_path_log_path_file_instead() {
        let invalid_path_file = "validation.rs";
        let result = valid_path_log(invalid_path_file);
        assert!(result.is_err());
        assert_eq!(result.err(), Some(GitError::InvalidLogDirectoryError));
    }

    #[test]
    fn test_invalid_path_log_empty_string() {
        let invalid_path_empty = "";
        let result = valid_path_log(invalid_path_empty);
        assert!(result.is_err());
        assert_eq!(result.err(), Some(GitError::InvalidLogDirectoryError));
    }

    #[test]
    fn test_valid_obj_id() {
        let valid_obj_id = "0123456789abcdef0123456789abcdef01234567";
        assert!(is_valid_obj_id(valid_obj_id));
    }

    #[test]
    fn test_invalid_obj_id() {
        let short_obj_id = "0123456789abcdef0123456789abcdef0123456";
        assert!(!is_valid_obj_id(short_obj_id));
    }

    #[test]
    fn valid_email_with_alphanumeric_chars_should_succeed() {
        let input = "username@example.com";
        let result = valid_email(input);
        assert!(result.is_ok());
    }

    #[test]
    fn valid_email_with_dashes_should_succeed() {
        let input = "user-name@example.com";
        let result = valid_email(input);
        assert!(result.is_ok());
    }

    #[test]
    fn valid_email_with_underscores_should_succeed() {
        let input = "user_name@example.com";
        let result = valid_email(input);
        assert!(result.is_ok());
    }

    #[test]
    fn valid_email_with_periods_should_succeed() {
        let input = "user.name@example.com";
        let result = valid_email(input);
        assert!(result.is_ok());
    }

    #[test]
    fn valid_email_with_multiple_domain_parts_should_succeed() {
        let input = "user.name@sub.example.co";
        let result = valid_email(input);
        assert!(result.is_ok());
    }

    #[test]
    fn valid_email_with_invalid_characters_should_fail() {
        let input = "user$@example.com";
        let result = valid_email(input);
        assert!(result.is_err());
    }

    #[test]
    fn valid_email_with_single_domain_part_should_fail() {
        let input = "username@localhost";
        let result = valid_email(input);
        assert!(result.is_err());
    }

    #[test]
    fn valid_email_with_missing_at_symbol_should_fail() {
        let input = "invalid_email.com";
        let result = valid_email(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_ipv4() {
        assert!(is_valid_ipv4("192.168.0.1"));
        assert!(is_valid_ipv4("0.0.0.0"));
        assert!(is_valid_ipv4("255.255.255.255"));
    }

    #[test]
    fn test_invalid_ipv4() {
        assert!(!is_valid_ipv4("256.0.0.1"));
        assert!(!is_valid_ipv4("192.168.0"));
        assert!(!is_valid_ipv4("invalid_ip_address"));
    }

    #[test]
    fn test_valid_ipv6() {
        assert!(is_ipv6("2001:0db8:85a3:0000:0000:8a2e:0370:7334"));
        assert!(is_ipv6("FF01:0:0:0:0:0:0:0"));
    }

    #[test]
    fn test_invalid_ipv6() {
        assert!(!is_ipv6("2001:0db8:85a3:0000:0000:8a2e:0370:7334:extra")); // Demasiadas secciones.
        assert!(!is_ipv6("2001:0db8:85a3:0000:0000:8a2e:0370:73")); // Sección faltante.
        assert!(!is_ipv6("2001:0db8:85a3:0000:0000:8z2e:0370:7334")); // Dígito no hexadecimal.
    }

    #[test]
    fn test_valid_local_part() {
        assert_eq!(is_valid_local_part("user123"), true);
        assert_eq!(is_valid_local_part("user_name"), true);
    }

    #[test]
    fn test_invalid_local_part() {
        assert_eq!(is_valid_local_part(""), false);
        assert_eq!(is_valid_local_part("user@example.com"), false);
        assert_eq!(is_valid_local_part("user name"), false);
        assert_eq!(is_valid_local_part("user!name"), false);
    }
}
