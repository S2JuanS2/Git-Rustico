use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::files::{open_file, read_file_string, create_file_replace};

/// Esta función se encarga de llamar al comando remote con los parametros necesarios.
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función remote
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_remote(args: Vec<&str>, client: Client) -> Result<String, GitError> {
    if args.len() > 3 {
        return Err(GitError::InvalidArgumentCountRemoteError);
    }
    let mut action = "none";
    let mut remote_name = "none";
    let mut remote_url = "none";
    if args.len() == 2 {
        action = args[0];
        remote_name = args[1];
    }
    if args.len() == 3 {
        action = args[0];
        remote_name = args[1];
        remote_url = args[2];
    }
    let directory = client.get_directory_path();
    let result = git_remote(directory, action, remote_name, remote_url)?;

    Ok(result)
}

/// Ejecuta la accion de remote en el repositorio local.
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'action': accion a realizar
/// 'remote_name': nombre del repositorio remoto
/// 'remote_url': url del repositorio remoto
pub fn git_remote(directory: &str, action: &str, remote_name: &str, remote_url: &str) -> Result<String, GitError> {
    let config_path = format!("{}/.git/config", directory);
    let config_file = open_file(&config_path)?;
    let config_content = read_file_string(config_file)?;
    let mut formatted_result = String::new();
    if action == "none" {
        let remotes = get_remotes(&config_content)?;
        for remote in remotes {
            formatted_result.push_str(format!("{}\n", remote).as_str());
        }
    }
    if action == "add" {
        add_remote(config_path.as_str(), &config_content, remote_name, remote_url)?;
    }
    if action == "rm" {
        remove_remote(config_path.as_str(), &config_content, remote_name)?;
    }
    
    Ok(formatted_result)
}

/// Obtiene los repositorios remotos del archivo de configuración.
/// ###Parametros:
/// 'config_path': ruta del archivo de configuración
fn get_remotes(config_content: &String) -> Result<Vec<String>, GitError> {
    let mut remotes = Vec::new();
    for line in config_content.lines() {
        if line.starts_with("[remote ") {
            if let Some(remote) = line.strip_prefix("[remote \"").and_then(|s| s.strip_suffix("\"]")) {
                remotes.push(remote.to_string());
            }
        }
    }
    Ok(remotes)
}

/// Agrega un repositorio remoto al archivo de configuración.
/// ###Parametros:
/// 'config_path': ruta del archivo de configuración
/// 'config_content': contenido del archivo de configuración
/// 'remote_name': nombre del repositorio remoto
/// 'remote_url': url del repositorio remoto
fn add_remote(config_path: &str, config_content: &String, remote_name: &str, remote_url: &str) -> Result<(), GitError> {
    let remote_exists = check_if_remote_exists(config_content.as_str(), remote_name);
    if remote_exists {
        return Err(GitError::RemoteAlreadyExistsError);
    }
    let remote = format!("[remote \"{}\"]\nurl = {}\nfetch = +refs/heads/*:refs/remotes/{}/*\n", remote_name, remote_url, remote_name);
    let new_config_content = format!("{}\n{}", config_content, remote);
    create_file_replace(config_path, new_config_content.as_str())?;
    
    Ok(())
}

/// Chequea si un repositorio remoto existe en el archivo de configuración.
/// ###Parametros:
/// 'config_content': contenido del archivo de configuración
/// 'remote_name': nombre del repositorio remoto
fn check_if_remote_exists(config_content: &str, remote_name: &str) -> bool {
    for line in config_content.lines() {
        if line.starts_with("[remote ") {
            if let Some(remote) = line.strip_prefix("[remote \"").and_then(|s| s.strip_suffix("\"]")) {
                if remote == remote_name {
                    return true
                }
            }
        }
    }
    false
}

/// Elimina un repositorio remoto del archivo de configuración.
/// ###Parametros:
/// 'config_path': ruta del archivo de configuración
/// 'config_content': contenido del archivo de configuración
/// 'remote_name': nombre del repositorio remoto
fn remove_remote(config_path: &str, config_content: &String, remote_name: &str) -> Result<(), GitError> {
    let remote_exists = check_if_remote_exists(config_content.as_str(), remote_name);
    if !remote_exists {
        return Err(GitError::RemoteDoesNotExistError);
    }

    let mut new_config_content = String::new();
    let mut in_remote_section = false;
    let mut lines_to_skip = 0;
    for line in config_content.lines() {
        if in_remote_section {
            lines_to_skip += 1;
            if lines_to_skip > 2 {
                in_remote_section = false;
            }
        }
        if line.starts_with("[remote \"") {
            if let Some(remote) = line.strip_prefix("[remote \"").and_then(|s| s.strip_suffix("\"]")) {
                if remote != remote_name {
                    new_config_content.push_str(format!("{}\n", line).as_str());
                } else {
                    in_remote_section = true;
                }
            }
        } else if !in_remote_section {
            new_config_content.push_str(format!("{}\n", line).as_str());
        }
    }
    create_file_replace(config_path, new_config_content.as_str())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use crate::{util::files::create_file_replace, commands::init::git_init};

    const CONFIG_CONTENT: &str = "[core]\n\
    repositoryformatversion = 0\n\
    filemode = false\n\
    bare = false\n\
    logallrefupdates = true\n\
    ignorecase = true\n\
[remote \"origin\"]\n\
    url = https://github.com/taller-1-fiuba-rust/23C2-Rusteam-Visionary.git\n\
    fetch = +refs/heads/*:refs/remotes/origin/*\n\
[remote \"upstream\"]\n\
    url = https://github.com/taller-1-fiuba-rust/23C2-Rusteam-Visionary.git\n\
    fetch = +refs/heads/*:refs/remotes/upstream/*\n\
[branch \"main\"]\n\
    remote = origin\n\
    merge = refs/heads/main\n\
[branch \"git_merge\"]\n\
    remote = origin\n\
    merge = refs/heads/git_merge";

    #[test]
    fn test_git_remote_empty() {
        let directory = "./test_remote";
        git_init(directory).expect("Error al iniciar el repositorio");
        let config_path = format!("{}/.git/config", directory);
        create_file_replace(config_path.as_str(), CONFIG_CONTENT).expect("Error al crear el archivo de configuración");
        let action = "none";
        let remote_name = "none";
        let remote_url = "none";
        let result = git_remote(directory, action, remote_name, remote_url);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "origin\nupstream\n".to_string());

        fs::remove_dir_all(directory).expect("Falló al remover el directorio temporal");
    }

    #[test]
    fn test_git_remote_add() {
        let directory = "./test_remote_add";
        git_init(directory).expect("Error al iniciar el repositorio");
        let config_path = format!("{}/.git/config", directory);
        create_file_replace(config_path.as_str(), CONFIG_CONTENT).expect("Error al crear el archivo de configuración");
        let action = "add";
        let remote_name = "test";
        let remote_url = "https://github.com/taller-1-fiuba-rust/23C2-Rusteam-Visionary.git";

        let result = git_remote(directory, action, remote_name, remote_url);

        assert!(result.is_ok());

        let config_path = open_file(&config_path).expect("Error al abrir el archivo de configuración");
        let config_content_after_remote_add = read_file_string(config_path).expect("Error al leer el archivo de configuración");

        assert!(config_content_after_remote_add.contains("[remote \"test\"]"));

        fs::remove_dir_all(directory).expect("Falló al remover el directorio temporal");
    }

    #[test]
    fn test_git_remote_rm() {
        let directory = "./test_remote_rm";
        git_init(directory).expect("Error al iniciar el repositorio");
        let config_path = format!("{}/.git/config", directory);
        create_file_replace(config_path.as_str(), CONFIG_CONTENT).expect("Error al crear el archivo de configuración");
        let action = "rm";
        let remote_name = "upstream";
        let remote_url = "none";

        let result = git_remote(directory, action, remote_name, remote_url);

        assert!(result.is_ok());

        let config_path = open_file(&config_path).expect("Error al abrir el archivo de configuración");
        let config_content_after_remote_rm = read_file_string(config_path).expect("Error al leer el archivo de configuración");

        assert!(!config_content_after_remote_rm.contains("[remote \"upstream\"]"));
        
        fs::remove_dir_all(directory).expect("Falló al remover el directorio temporal");
    }

}