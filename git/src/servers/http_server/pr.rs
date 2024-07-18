use serde_json::Value;
use crate::{servers::errors::ServerError, util::logger::log_message};
use std::sync::{mpsc::Sender, Arc, Mutex};


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

    pub fn create_pull_requests(&self,repo_name: &str, _src: &String,tx: &Arc<Mutex<Sender<String>>>) -> Result<String, ServerError> {
        let message = format!("POST request to path: /repos/{}/pulls", repo_name);
        println!("{}", message);
        log_message(&tx, &message);
        Ok("Pull request created".to_string())
    }

    pub fn list_pull_request(&self,repo_name: &str, _src: &String, tx: &Arc<Mutex<Sender<String>>>) -> Result<String, ServerError> {
        let message = format!("GET request to path: /repos/{}/pulls", repo_name);
        println!("{}", message);
        log_message(&tx, &message);
        Ok("Pull request listed".to_string())
    }

    pub fn get_pull_request(&self,repo_name: &str, pull_number: &str, _src: &String, tx: &Arc<Mutex<Sender<String>>>) -> Result<String, ServerError> {
        let message = format!("GET request to path: /repos/{}/pulls/{}", repo_name, pull_number);
        println!("{}", message);
        log_message(&tx, &message);
        Ok("Pull request retrieved".to_string())
    }

    pub fn list_commits(&self,repo_name: &str, pull_number: &str, _src: &String, tx: &Arc<Mutex<Sender<String>>>) -> Result<String, ServerError> {
        let message = format!("GET request to path: /repos/{}/pulls/{}/commits", repo_name, pull_number);
        println!("{}", message);
        log_message(&tx, &message);
        Ok("Commits listed".to_string())
    }

    pub fn merge_pull_request(&self,repo_name: &str, pull_number: &str, _src: &String, tx: &Arc<Mutex<Sender<String>>>) -> Result<String, ServerError> {
        let message = format!("PUT request to path: /repos/{}/pulls/{}/merge", repo_name, pull_number);
        println!("{}", message);
        log_message(&tx, &message);
        Ok("Pull request merged".to_string())
    }

    pub fn modify_pull_request(&self,repo_name: &str, pull_number: &str, _src: &String, tx: &Arc<Mutex<Sender<String>>>) -> Result<String, ServerError> {
        let message = format!("PATCH request to path: /repos/{}/pulls/{}", repo_name, pull_number);
        println!("{}", message);
        log_message(&tx, &message);
        Ok("Pull request modified".to_string())
    }
}