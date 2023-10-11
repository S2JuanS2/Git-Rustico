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
/// use git::is_valid_ip;
/// use git::GitError;
///
/// match is_valid_ip("192.168.1.1".to_string()) {
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
pub fn is_valid_ip(input: String) -> Result<String, GitError> {
    if is_valid_ipv4(&input) || is_ipv6(&input)
    {
        return Ok(input);
    }
    return Err(GitError::InvalidIpError);
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
/// use git::valid_path_log;
/// use git::GitError;
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
/// use git::valid_email;
/// use git::GitError;
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

    if local_part.is_empty() || domain_part.is_empty() {
        return Err(GitError::InvalidUserMailError);
    }

    // Verificar que el local part no contenga caracteres inválidos
    for c in local_part.chars() {
        if !c.is_alphanumeric() && c != '.' && c != '-' && c != '_' {
            return Err(GitError::InvalidUserMailError);
        }
    }

    // Verificar que el dominio tenga al menos un punto y no contenga caracteres inválidos
    let domain_parts: Vec<&str> = domain_part.split('.').collect();
    if domain_parts.len() < 2 {
        return Err(GitError::InvalidUserMailError);
    }

    for part in domain_parts {
        for c in part.chars() {
            if !c.is_alphanumeric() && c != '-' {
                return Err(GitError::InvalidUserMailError);
            }
        }
    }

    Ok(input.to_string())
}



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

fn is_ipv6(ip: &str) -> bool {
    let parts: Vec<&str> = ip.split(':').collect();

    if parts.len() != IPV6_SECTIONS {
        return false; 
    }

    for part in parts {
        if part.len() != IPV4_SECTION_LENGTH {
            return false; 
        }

        if u16::from_str_radix(part, 16).is_err() {
            return false;
        }

        let value = match u16::from_str_radix(part, 16) {
            Ok(n) => n,
            Err(_) => return false,
        } ;

        if value > IPV6_MAX {
            return false;
        }
    }

    true
}