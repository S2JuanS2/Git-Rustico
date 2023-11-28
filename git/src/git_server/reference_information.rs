#[derive(Debug)]
pub struct ReferenceInformation {
    remote_commit: String,
    local_commit: Option<String>,
    pub confirmed: bool,
}

impl ReferenceInformation {
    pub fn new(remote_commit: &str, local_commit: Option<String>) -> Self {
        ReferenceInformation {
            remote_commit: remote_commit.to_string(),
            local_commit,
            confirmed: false,
        }
    }

    // fn update_remote_commit(&mut self, remote_commit: &str) {
    //     self.remote_commit = remote_commit.to_string();
    // }

    pub fn update_local_commit(&mut self, local_commit: Option<String>) {
        self.local_commit = local_commit;
    }

    pub fn get_remote_commit(&self) -> &str {
        &self.remote_commit
    }

    pub fn get_local_commit(&self) -> Option<&str> {
        if let Some(local_commit) = &self.local_commit {
            Some(local_commit)
        } else {
            None
        }
    }

    pub fn confirm_local_commit(&mut self) {
        self.confirmed = true;
    }
}