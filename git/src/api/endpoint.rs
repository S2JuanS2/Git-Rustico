/// Crea la URL para crear una solicitud de extracción en un repositorio específico.
///
/// # Arguments
///
/// * `name_repo` - El nombre del repositorio.
///
/// # Returns
///
/// Un `String` que representa la URL para crear una solicitud de extracción en el repositorio especificado.
///
pub fn create_pull_requests(name_repo: String) -> String{
    // /repos/{repo}/pulls
    format!("repos/{}/pulls", name_repo)
}

/// Crea la URL para listar todas las solicitudes de extracción en un repositorio específico.
///
/// # Arguments
///
/// * `name_repo` - El nombre del repositorio.
///
/// # Returns
///
/// Un `String` que representa la URL para listar todas las solicitudes de extracción en el repositorio especificado.
///
pub fn list_pull_requests(name_repo: String) -> String{
    // /repos/{repo}/pulls
    format!("repos/{}/pulls", name_repo)
}

/// Crea la URL para obtener una solicitud de extracción específica en un repositorio.
///
/// # Arguments
///
/// * `name_repo` - El nombre del repositorio.
/// * `number` - El número de la solicitud de extracción.
///
/// # Returns
///
/// Un `String` que representa la URL para obtener una solicitud de extracción específica en el repositorio especificado.
///
pub fn get_pull_request(name_repo: String, number: u32) -> String{
    // /repos/{repo}/pulls/{number}
    format!("repos/{}/pulls/{}", name_repo, number)
}

/// Crea la URL para listar todos los commits en una solicitud de extracción específica en un repositorio.
///
/// # Arguments
///
/// * `name_repo` - El nombre del repositorio.
/// * `number` - El número de la solicitud de extracción.
///
/// # Returns
///
/// Un `String` que representa la URL para listar todos los commits en una solicitud de extracción específica en el repositorio especificado.
///
pub fn list_commits(name_repo: String, number: u32) -> String{
    // /repos/{repo}/pulls/{number}/commits
    format!("repos/{}/pulls/{}/commits", name_repo, number)
}

/// Crea la URL para fusionar una solicitud de extracción específica en un repositorio.
///
/// # Arguments
///
/// * `name_repo` - El nombre del repositorio.
/// * `number` - El número de la solicitud de extracción.
///
/// # Returns
///
/// Un `String` que representa la URL para fusionar una solicitud de extracción específica en el repositorio especificado.
///
pub fn merge_pull_request(name_repo: String, number: u32) -> String{
    // /repos/{repo}/pulls/{number}/merge
    format!("repos/{}/pulls/{}/merge", name_repo, number)
}

/// Crea la URL para actualizar una solicitud de extracción específica en un repositorio.
///
/// # Arguments
///
/// * `name_repo` - El nombre del repositorio.
/// * `number` - El número de la solicitud de extracción.
///
/// # Returns
///
/// Un `String` que representa la URL para actualizar una solicitud de extracción específica en el repositorio especificado.
///
pub fn update_pull_request(name_repo: String, number: u32) -> String{
    // /repos/{repo}/pulls/{number}
    format!("repos/{}/pulls/{}", name_repo, number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_pull_requests() {
        let name_repo = "my_repo".to_string();
        let expected_result = "repos/my_repo/pulls".to_string();
        assert_eq!(create_pull_requests(name_repo), expected_result);
    }

    #[test]
    fn test_list_pull_requests() {
        let name_repo = "my_repo".to_string();
        let expected_result = "repos/my_repo/pulls".to_string();
        assert_eq!(list_pull_requests(name_repo), expected_result);
    }

    #[test]
    fn test_get_pull_request() {
        let name_repo = "my_repo".to_string();
        let number = 123;
        let expected_result = "repos/my_repo/pulls/123".to_string();
        assert_eq!(get_pull_request(name_repo, number), expected_result);
    }

    #[test]
    fn test_list_commits() {
        let name_repo = "my_repo".to_string();
        let number = 123;
        let expected_result = "repos/my_repo/pulls/123/commits".to_string();
        assert_eq!(list_commits(name_repo, number), expected_result);
    }

    #[test]
    fn test_merge_pull_request() {
        let name_repo = "my_repo".to_string();
        let number = 123;
        let expected_result = "repos/my_repo/pulls/123/merge".to_string();
        assert_eq!(merge_pull_request(name_repo, number), expected_result);
    }

    #[test]
    fn test_update_pull_request() {
        let name_repo = "my_repo".to_string();
        let number = 123;
        let expected_result = "repos/my_repo/pulls/123".to_string();
        assert_eq!(update_pull_request(name_repo, number), expected_result);
    }
}
