use crate::consts::{APPLICATION_JSON, APPLICATION_XML, APPLICATION_YAML, TEXT_XML, TEXT_YAML};

use super::pr::{CommitsPr, PullRequest};


#[derive(Debug, PartialEq)]
pub enum Model {
    // PullRequest(HttpBody),
    // CommitsPr(HttpBody),
    ListPullRequest(Vec<PullRequest>),
    ListCommits(Vec<CommitsPr>),
    Message(String),
    // Empty,
}

impl Model {
    pub fn to_string(&self, content_type: &str) -> String {
        match self {
            Model::ListPullRequest(v) => list_pull_request_to_string(v, content_type),
            Model::ListCommits(v) => list_commits_to_string(v, content_type),
            Model::Message(s) => message_to_string(s, content_type),
        }
    }

}

fn pull_request_to_string(pr: &PullRequest, content_type: &str) -> String {
    let mut result = String::new();
    let id = pr.id.unwrap_or_default();
    let owner = pr.owner.clone().unwrap_or_default();
    let title = pr.title.clone().unwrap_or_default();
    let body = pr.body.clone().unwrap_or_default();
    let state = pr.state.clone().unwrap_or_default();
    let base = pr.base.clone().unwrap_or_default();
    let head = pr.head.clone().unwrap_or_default();
    let repo = pr.repo.clone().unwrap_or_default();
    let mergeable = pr.mergeable.clone().unwrap_or_default();
    let changed_files = convert_vector_in_string(pr.changed_files.clone().unwrap_or_default());
    let amount_commits = pr.amount_commits.unwrap_or_default();
    let commits = convert_vector_in_string(pr.commits.clone().unwrap_or_default());


    match content_type {
        APPLICATION_JSON => {
            result.push_str(&format!("{{\t\"id\": {},\n\t\"owner\": \"{}\",\n\t\"title\": \"{}\",\n\t\"body\": \"{}\",\n\t\"state\": \"{}\",\n\t\"base\": \"{}\",\n\t\"head\": \"{}\",\n\t\"repo\": \"{}\",\n\t\"mergeable\": {},\n\t\"changed_files\": {},\n\t\"amount_commits\": {},\n\t\"commits\": {}}}", id, owner, title, body, state, base, head, repo, mergeable, changed_files, amount_commits, commits));
        }
        TEXT_XML | APPLICATION_XML => { 
                        result.push_str(&format!(
                "<pull_request>\n\
                \t<id>{}</id>\n\
                \t<owner>{}</owner>\n\
                \t<title>{}</title>\n\
                \t<body>{}</body>\n\
                \t<state>{}</state>\n\
                \t<base>{}</base>\n\
                \t<head>{}</head>\n\
                \t<repo>{}</repo>\n\
                \t<mergeable>{}</mergeable>\n\
                \t<changed_files>{}</changed_files>\n\
                \t<amount_commits>{}</amount_commits>\n\
                \t<commits>{}</commits>\n\
                </pull_request>",
                id,
                escape_xml(&owner),
                escape_xml(&title),
                escape_xml(&body),
                escape_xml(&state),
                escape_xml(&base),
                escape_xml(&head),
                escape_xml(&repo),
                mergeable,
                escape_xml(&changed_files),
                amount_commits,
                escape_xml(&commits)
            ));
        }
        TEXT_YAML | APPLICATION_YAML => {
            result.push_str(&format!(
                "id: {}\n\
                owner: \"{}\"\n\
                title: \"{}\"\n\
                body: |\n  {}\n\
                state: \"{}\"\n\
                base: \"{}\"\n\
                head: \"{}\"\n\
                repo: \"{}\"\n\
                mergeable: {}\n\
                changed_files: [{}]\n\
                amount_commits: {}\n\
                commits: [{}]",
                id,
                owner,
                title,
                body.replace("\n", "\n  "), // Indentar las líneas del cuerpo
                state,
                base,
                head,
                repo,
                mergeable,
                changed_files,
                amount_commits,
                commits
            ));
        }
        _ => return "".to_string(),
    };

    result
}


fn commits_to_string(commit: &CommitsPr, content_type: &str) -> String {
    let mut result = String::new();
    let sha_1 = commit.sha_1.clone();
    let tree_hash = commit.tree_hash.clone();
    let parent = commit.parent.clone();
    let author_name = commit.author_name.clone();
    let author_email = commit.author_email.clone();
    let committer_name = commit.committer_name.clone();
    let committer_email = commit.committer_email.clone();
    let message = commit.message.clone();
    let date = commit.date.clone();

    match content_type {
        APPLICATION_JSON => {
            result.push_str(&format!("{{\t\"sha_1\": \"{}\",\n\t\"tree_hash\": \"{}\",\n\t\"parent\": \"{}\",\n\t\"author_name\": \"{}\",\n\t\"author_email\": {},\n\t\"committer_name\": \"{}\",\n\t\"committer_email\": {},\n\t\"message\": \"{}\",\n\t\"date\": \"{}\"}}", sha_1, tree_hash, parent, author_name, author_email, committer_name, committer_email, message, date));
        }
        TEXT_XML | APPLICATION_XML => { 
            let author_email = escape_xml(author_email.as_str());
            let committer_email = escape_xml(committer_email.as_str());
            result.push_str(&format!(
                "<commit>\n\
                \t<sha_1>{}</sha_1>\n\
                \t<tree_hash>{}</tree_hash>\n\
                \t<parent>{}</parent>\n\
                \t<author_name>{}</author_name>\n\
                \t<author_email>\"{}\"</author_email>\n\
                \t<committer_name>{}</committer_name>\n\
                \t<committer_email>\"{}\"</committer_email>\n\
                \t<message>{}</message>\n\
                \t<date>{}</date>\n\
                </commit>",
                sha_1, tree_hash, parent, author_name, author_email, committer_name, committer_email, message, date
            ))
        }
        TEXT_YAML | APPLICATION_YAML => {
            result.push_str(&format!(
                "  - sha_1: \"{}\"\n\
                 tree_hash: \"{}\"\n\
                 parent: \"{}\"\n\
                 author_name: \"{}\"\n\
                 author_email: \"{}\"\n\
                 committer_name: \"{}\"\n\
                 committer_email: \"{}\"\n\
                 message: \"{}\"\n\
                 date: {}\n",
                sha_1, tree_hash, parent, author_name, author_email, committer_name, committer_email, message, date
            ));
        }
        _ => return "".to_string(),
    };
    result
}

fn convert_vector_in_string(vec: Vec<String>) -> String {
    let mut result = String::new();
    result.push_str("[");
    for (i, item) in vec.iter().enumerate() {
        result.push_str(format!("\"{}\"", item).as_str());
        if i < vec.len() - 1 {
            result.push_str(", ");
        }
    }
    result.push_str("]");
    result
}


fn list_pull_request_to_string(prs: &Vec<PullRequest>, content_type: &str) -> String {
    let mut result = String::new();
    match content_type {
        APPLICATION_JSON => {
            result.push_str("{\n");
            for (i, pr) in prs.iter().enumerate() {
                result.push_str(&pull_request_to_string(pr, content_type));
                if i < prs.len() - 1 {
                    result.push_str(", ");
                }
            }
            result.push_str("\n}");
        }
        TEXT_XML | APPLICATION_XML => {
            result.push_str("<prs>");
            for pr in prs.iter() {
                result.push_str(&pull_request_to_string(pr, content_type));
            }
            result.push_str("</prs>");
        }
        TEXT_YAML | APPLICATION_YAML => {
            result.push_str("prs:\n");
            for pr in prs.iter() {
                result.push_str(&pull_request_to_string(pr, content_type));
            }
        }
        _ => return "".to_string(),
    };
    result
}

fn list_commits_to_string(commits: &Vec<CommitsPr>, content_type: &str) -> String {
    let mut result = String::new();
    match content_type {
        APPLICATION_JSON => {
            result.push_str("{\n");
            for (i, commit) in commits.iter().enumerate() {
                result.push_str(&commits_to_string(commit, content_type));
                if i < commits.len() - 1 {
                    result.push_str(", ");
                }
            }
            result.push_str("\n}");
        }
        TEXT_XML | APPLICATION_XML => {
            result.push_str("<commits>");
            for commit in commits.iter() {
                result.push_str(&commits_to_string(commit, content_type));
            }
            result.push_str("</commits>");
        }
        TEXT_YAML | APPLICATION_YAML => {
            result.push_str("commits:\n");
            for commit in commits.iter() {
                result.push_str(&commits_to_string(commit, content_type));
            }
        }
        _ => return "".to_string(),
    };
    result
}

fn message_to_string(message: &str, content_type: &str) -> String {
    let mut result = String::new();
    match content_type {
        APPLICATION_JSON => {
            result.push_str(&format!("{{message: {}}}", message));
        }
        TEXT_XML | APPLICATION_XML => {
            result.push_str(&format!("<message>{}</message>", message));
        }
        TEXT_YAML | APPLICATION_YAML => {
            result.push_str(&format!("message: {}", message));
        }
        _ => return "".to_string(),
    };
    result
}


fn escape_xml(input: &str) -> String {
    input
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
}