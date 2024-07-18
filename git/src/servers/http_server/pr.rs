use serde_json::Value;

use crate::servers::errors::ServerError;


#[derive(Debug)]
pub struct PullRequest {
    pub owner: String,
    pub repo: String,
    pub title: Option<String>,
    pub body: Option<String>,
    pub head: Option<String>,
    pub base: Option<String>,
}

impl PullRequest {
    pub fn from_json(json: &Value) -> Result<Self, ServerError> {
        let owner = json["owner"].as_str()
            .ok_or(ServerError::HttpNoOwnerFound)?
            .to_string();

        let repo = json["repo"].as_str()
            .ok_or(ServerError::HttpNoRepoFound)?
            .to_string();

        Ok(PullRequest {
            owner,
            repo,
            title: json["title"].as_str().map(ToString::to_string),
            body: json["body"].as_str().map(ToString::to_string),
            head: json["head"].as_str().map(ToString::to_string),
            base: json["base"].as_str().map(ToString::to_string),
        })
    }
}