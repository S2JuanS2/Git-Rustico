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
