use crate::models::client::Client;

pub struct PrInfo {
    pub owner: String,
    pub repo: String,
    pub title: String,
    pub body: String,
    pub head: String,
    pub base: String,
}

impl PrInfo {
    pub fn new(client: &Client, title: &str, body: &str, head: &str, base: &str) -> PrInfo {
        PrInfo {
            owner: client.get_name().to_string(),
            repo: client.get_directory_path().to_string(),
            title: title.to_string(),
            body: body.to_string(),
            head: head.to_string(),
            base: base.to_string(),
        }
    }
}