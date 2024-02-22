use super::endpoint::{create_pull_requests, get_pull_request, list_commits, list_pull_requests, merge_pull_request, update_pull_request};

enum PrOperation {
    Create,
    List,
    Get,
    ListCommits,
    Merge,
    Update, //Modificar
}

impl PrOperation {
    pub fn get_endpoint(&self, name_repo: String, number: u32) -> String {
        match self {
            PrOperation::Create => create_pull_requests(name_repo),
            PrOperation::List => list_pull_requests(name_repo),
            PrOperation::Get => get_pull_request(name_repo, number),
            PrOperation::ListCommits => list_commits(name_repo, number),
            PrOperation::Merge => merge_pull_request(name_repo, number),
            PrOperation::Update => update_pull_request(name_repo, number),
        }
    }
}