use serde_json::Value;

use crate::servers::errors::ServerError;

pub struct PullRequest {
    pub owner: String,
    pub repo: String,
    pub title: Option<String>,
    pub body: Option<String>,
    pub head: Option<String>,
    pub base: Option<String>,
}

impl PullRequest {
    pub fn from_json(json: Value) -> Result<Self, ServerError> {
        let owner = match json["owner"].as_str()
        {
            Some(owner) => owner.to_string(),
            None => return Err(ServerError::HttpNoOwnerFound),
        };
        let repo = match json["repo"].as_str()
        {
            Some(repo) => repo.to_string(),
            None => return Err(ServerError::HttpNoRepoFound),
        };

        Ok(PullRequest {
            owner: owner,
            repo: repo,
            title: json["title"].as_str().map(|s| s.to_string()),
            body: json["body"].as_str().map(|s| s.to_string()),
            head: json["head"].as_str().map(|s| s.to_string()),
            base: json["base"].as_str().map(|s| s.to_string()),
        })
    }
}