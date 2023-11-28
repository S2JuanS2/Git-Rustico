#[derive(Debug)]
pub struct ReferenceInformation {
    pub last_commit: String,
    pub current_commit: Option<String>,
    pub confirmed: bool,
}

impl ReferenceInformation {
    pub fn new(last_commit: &str, current_commit: Option<String>) -> Self {
        ReferenceInformation {
            last_commit: last_commit.to_string(),
            current_commit,
            confirmed: false,
        }
    }

    // fn update_last_commit(&mut self, last_commit: &str) {
    //     self.last_commit = last_commit.to_string();
    // }

    pub fn update_current_commit(&mut self, current_commit: Option<String>) {
        self.current_commit = current_commit;
    }
}