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
}