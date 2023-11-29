
/// Estructura que almacena información sobre una referencia en un repositorio Git.
/// Miembros:
/// - `remote_commit`: Commit remoto al que apunta la referencia.
/// - `local_commit`: Commit local al que apunta la referencia (puede ser `None`).
/// - `confirmed`: Indica si el commit local ha sido verificado o confirmado.
/// 
#[derive(Debug)]
pub struct ReferenceInformation {
    remote_commit: String,
    local_commit: Option<String>,
    confirmed: bool,
}

impl ReferenceInformation {
    /// Crea una nueva instancia de `ReferenceInformation`.
    ///
    /// # Argumentos
    ///
    /// * `remote_commit`: Commit remoto al que apunta la referencia.
    /// * `local_commit`: Commit local al que apunta la referencia (puede ser `None`).
    ///
    pub fn new(remote_commit: &str, local_commit: Option<String>) -> Self {
        ReferenceInformation {
            remote_commit: remote_commit.to_string(),
            local_commit,
            confirmed: false,
        }
    }

    /// Actualiza el commit local almacenado en la estructura `ReferenceInformation`.
    ///
    /// # Argumentos
    ///
    /// * `local_commit`: Nuevo commit local al que se actualizará la referencia (puede ser `None`).
    ///
    pub fn update_local_commit(&mut self, local_commit: Option<String>) {
        self.local_commit = local_commit;
    }

    /// Obtiene el commit remoto al que apunta la referencia.
    /// 
    pub fn get_remote_commit(&self) -> &str {
        &self.remote_commit
    }

    /// Obtiene el commit local al que apunta la referencia, si está presente.
    ///
    pub fn get_local_commit(&self) -> Option<&str> {
        if let Some(local_commit) = &self.local_commit {
            Some(local_commit)
        } else {
            None
        }
    }
    /// Confirma que el commit local ha sido verificado o confirmado.
    ///
    pub fn confirm_local_commit(&mut self) {
        self.confirmed = true;
    }

    /// Indica si el commit local ha sido verificado o confirmado.
    /// 
    pub fn is_confirmed(&self) -> bool {
        self.confirmed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_local_commit() {
        let mut reference = ReferenceInformation::new("abc123", Some("def456".to_string()));
        reference.update_local_commit(Some("ghi789".to_string()));

        assert_eq!(reference.get_local_commit(), Some("ghi789"));
    }

    #[test]
    fn test_get_remote_commit() {
        let reference = ReferenceInformation::new("abc123", Some("def456".to_string()));

        assert_eq!(reference.get_remote_commit(), "abc123");
    }

    #[test]
    fn test_get_local_commit() {
        let reference = ReferenceInformation::new("abc123", Some("def456".to_string()));

        assert_eq!(reference.get_local_commit(), Some("def456"));
    }

    #[test]
    fn test_get_local_commit_none() {
        let reference = ReferenceInformation::new("abc123", None);

        assert_eq!(reference.get_local_commit(), None);
    }

    #[test]
    fn test_confirm_local_commit() {
        let mut reference = ReferenceInformation::new("abc123", Some("def456".to_string()));
        reference.confirm_local_commit();

        assert_eq!(reference.confirmed, true);
    }
}
