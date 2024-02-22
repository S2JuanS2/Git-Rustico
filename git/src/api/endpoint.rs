pub fn create_pull_requests(name_repo: String) -> String{
    // /repos/{repo}/pulls
    format!("repos/{}/pulls", name_repo)
}

pub fn list_pull_requests(name_repo: String) -> String{
    // /repos/{repo}/pulls
    format!("repos/{}/pulls", name_repo)
}

pub fn get_pull_request(name_repo: String, number: u32) -> String{
    // /repos/{repo}/pulls/{number}
    format!("repos/{}/pulls/{}", name_repo, number)
}

pub fn list_commits(name_repo: String, number: u32) -> String{
    // /repos/{repo}/pulls/{number}/commits
    format!("repos/{}/pulls/{}/commits", name_repo, number)
}

pub fn merge_pull_request(name_repo: String, number: u32) -> String{
    // /repos/{repo}/pulls/{number}/merge
    format!("repos/{}/pulls/{}/merge", name_repo, number)
}

pub fn update_pull_request(name_repo: String, number: u32) -> String{
    // /repos/{repo}/pulls/{number}
    format!("repos/{}/pulls/{}", name_repo, number)
}
