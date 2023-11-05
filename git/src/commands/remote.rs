use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::files::{open_file, read_file_string, create_file_replace};

/// Esta función se encarga de llamar al comando remote con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función remote
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_remote(args: Vec<&str>, client: Client) -> Result<(), GitError> {
    if args.len() > 3 {
        return Err(GitError::InvalidArgumentCountRemoteError);
    }
    let mut action = "none";
    let mut remote_name = "none";
    let mut remote_url = "none";
    if args.len() == 3 {
        action = args[0];
        remote_name = args[1];
        remote_url = args[2];
    }
    let directory = client.get_directory_path();
    git_remote(directory, action, remote_name, remote_url)
}

/// ejecuta la accion de remote en el repositorio local
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'action': accion a realizar
/// 'remote_name': nombre del repositorio remoto
/// 'remote_url': url del repositorio remoto
pub fn git_remote(directory: &str, action: &str, remote_name: &str, remote_url: &str) -> Result<(), GitError> {
    let config_path = format!("{}/.git/config", directory);
    if action == "none" {
        get_remotes(config_path.as_str())?;
    }
    if action == "add" {
        add_remote(config_path.as_str(), remote_name, remote_url)?;
    }
    
    Ok(())
}

/// Obtiene los repositorios remotos del archivo de configuración
/// ###Parametros:
/// 'config_path': ruta del archivo de configuración
fn get_remotes(config_path: &str) -> Result<Vec<String>, GitError> {
    let config_file = open_file(config_path)?;
    let config_content = read_file_string(config_file)?;
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

fn add_remote(config_path: &str, remote_name: &str, remote_url: &str) -> Result<(), GitError> {
    let config_file = open_file(config_path)?;
    let config_content = read_file_string(config_file)?;
    let remote_exists = check_if_remote_exists(config_content.as_str(), remote_name);
    if remote_exists {
        return Err(GitError::RemoteAlreadyExistsError);
    }
    let remote = format!("[remote \"{}\"]\nurl = {}\nfetch = +refs/heads/*:refs/remotes/{}/*\n", remote_name, remote_url, remote_name);
    let new_config_content = format!("{}\n{}", config_content, remote);
    create_file_replace(config_path, new_config_content.as_str())?;
    
    Ok(())
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{util::files::create_file, commands::init::git_init};

    #[test]
    fn test_git_remote_empty() {
        let directory = "./test_remote";
        git_init(directory).expect("Error al iniciar el repositorio");
        let config_path = format!("{}/.git/config", directory);
        let config_content = "[core]\n\
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
        create_file(config_path.as_str(), config_content).expect("Error al crear el archivo de configuración");
        let action = "none";
        let remote_name = "none";
        let remote_url = "none";
        git_remote(directory, action, remote_name, remote_url).expect("Error al ejecutar git remote");
        
    }

    #[test]
    fn test_git_remote_add() {
        let directory = "./test_remote_add";
        git_init(directory).expect("Error al iniciar el repositorio");
        let config_path = format!("{}/.git/config", directory);
        let config_content = "[core]\n\
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
        create_file(config_path.as_str(), config_content).expect("Error al crear el archivo de configuración");
        let action = "add";
        let remote_name = "test";
        let remote_url = "https://github.com/taller-1-fiuba-rust/23C2-Rusteam-Visionary.git";

        git_remote(directory, action, remote_name, remote_url).expect("Error al ejecutar git remote");
    }
}