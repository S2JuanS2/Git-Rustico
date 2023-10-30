use crate::errors::GitError;
use crate::models::client::Client;

/// Esta función se encarga de llamar al comando remote con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función remote
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_remote(args: Vec<&str>, client: Client) -> Result<(), GitError> {
    if args.len() > 3 {
        return Err(GitError::InvalidArgumentCountRemoteError);
    }
    let directory = client.get_directory_path();
    let action = args[0];
    let remote_name = args[1];
    let remote_url = args[2];
    git_remote(&directory, action, remote_name, remote_url)
}

/// ejecuta la accion de remote en el repositorio local
/// ###Parametros:
/// 'directory': directorio del repositorio local
/// 'action': accion a realizar
/// 'remote_name': nombre del repositorio remoto
/// 'remote_url': url del repositorio remoto
pub fn git_remote(
    _directory: &str,
    _action: &str,
    _remote_name: &str,
    _remote_url: &str,
) -> Result<(), GitError> {
    Ok(())
}
