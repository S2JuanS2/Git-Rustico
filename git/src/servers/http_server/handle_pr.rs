use crate::{servers::errors::ServerError, util::logger::log_message};
use super::{http_request::HttpRequest, pr::PullRequest};
use std::sync::{mpsc::Sender, Arc, Mutex};


pub fn handle_get_request(request: &HttpRequest, pr: &PullRequest, src: &String, tx: &Arc<Mutex<Sender<String>>>) -> Result<String, ServerError> {
    let path_segments: Vec<&str> = request.get_path().split('/').collect();
    match path_segments.as_slice() {
        ["repos", repo_name, "pulls"] => {
            return pr.list_pull_request(repo_name, src, tx);
        },
        ["repos", repo_name, "pulls", pull_number] => {
            return pr.get_pull_request(repo_name, pull_number, src, tx);
        },
        ["repos", repo_name, "pulls", pull_number, "commits"] => {
            return pr.list_commits(repo_name, pull_number, src, tx);
        },
        _ => {
            return Err(ServerError::MethodNotAllowed);
        }
    }
}

pub fn handle_post_request(request: &HttpRequest, pr: &PullRequest, src: &String, tx: &Arc<Mutex<Sender<String>>>) -> Result<String, ServerError> {
    let path_segments: Vec<&str> = request.get_path().split('/').collect();
    match path_segments.as_slice() {
        ["repos", repo_name, "pulls"] => {
            return pr.create_pull_requests(repo_name, src, tx);
        }
        _ => {
            return Err(ServerError::MethodNotAllowed);
        }
    }
}

pub fn handle_put_request(request: &HttpRequest, tx: &Arc<Mutex<Sender<String>>>) -> Result<String, ServerError> {
    let message = request.body["message"].as_str().unwrap_or("No message");
    let message = message.to_string();
    let message = format!("PUT request to path: {} with message: {}", request.path, message);
    log_message(&tx, &message);
    println!("{}", message);
    Ok(message)
}

pub fn handle_patch_request(request: &HttpRequest, tx: &Arc<Mutex<Sender<String>>>) -> Result<String, ServerError> {
    let message = request.body["message"].as_str().unwrap_or("No message");
    let message = message.to_string();
    let message = format!("PATCH request to path: {} with message: {}", request.path, message);
    log_message(&tx, &message);
    println!("{}", message);
    Ok(message)
}