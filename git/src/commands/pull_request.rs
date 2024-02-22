use crate::api::http_operation::HttpOperation;

pub struct PRInfo {
    pub owner: String,
    pub repo: String,
    pub title: String,
    pub body: String,
    pub head: String,
    pub base: String,
}
pub fn handle_pr(pr_info: PRInfo, operation: HttpOperation) -> Result<String, CommandsError> {
}

impl PRInfo {
    pub fn new(title: &str, body: &str, head: &str, base: &str) -> PRInfo {
        let owner = "owner";
        let repo = "repo";
        PRInfo {
            owner: owner.to_string(),
            repo: repo.to_string(),
            title: title.to_string(),
            body: body.to_string(),
            head: head.to_string(),
            base: base.to_string(),
        }
    }
}