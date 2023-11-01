#[derive(Clone)]
pub struct Client {
    name: String,
    email: String,
    ip: String,
    directory_path: String,
}
impl Client {
    pub fn new(name: String, email: String, ip: String, directory_path: String) -> Client {
        Client {
            name,
            email,
            ip,
            directory_path,
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
}
