#[derive(Clone)]
pub struct Client {
    name: String,
    email: String,
    ip: String,
    directory_path: String,
    path_log: String,
}
impl Client {
    pub fn new(
        name: String,
        email: String,
        ip: String,
        directory_path: String,
        path_log: String,
    ) -> Client {
        Client {
            name,
            email,
            ip,
            directory_path,
            path_log,
        }
    }

    pub fn get_ip(&self) -> &str {
        &self.ip
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_email(&self) -> &str {
        &self.email
    }

    pub fn get_directory_path(&self) -> &str {
        &self.directory_path
    }

    pub fn get_path_log(&self) -> &str {
        &self.path_log
    }

    pub fn set_directory_path(&mut self, new_path: String) {
        self.directory_path = new_path;
    }
}
