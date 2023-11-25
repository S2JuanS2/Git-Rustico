pub enum Label {
    NotForMerge,
    None,
}

pub struct FetchHead {
    pub commit_hash: String,
    pub branch_name: String,
    pub label: Label,
}

impl FetchHead {
    pub fn new(commit_hash: String, branch_name: String, label: String) -> Self {
        let label = match label.as_str() {
            "not-for-merge" => Label::NotForMerge,
            _ => Label::None,
        };
        FetchHead {
            commit_hash,
            branch_name,
            label,
        }
    }

    
}