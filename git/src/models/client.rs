#[derive(Clone)]
pub struct Client {
    ip: String,
    directory_path: String,
}
impl Client {
    pub fn new(ip: String, directory_path: String) -> Client {
        Client { ip, directory_path }
    }

    pub fn get_ip(self) -> String {
        self.ip
    }

    pub fn get_directory_path(self) -> String {
        self.directory_path
    }
}
