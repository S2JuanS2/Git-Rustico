pub const ENTRY_CONSOLE: &str = "entry_console";
pub const ENTRY_BRANCH: &str = "entry_branch";
pub const ENTRY_CHECKOUT: &str = "entry_checkout";
pub const ENTRY_ADD_RM: &str = "entry_add_rm";
pub const ENTRY_COMMIT: &str = "entry_commit";
pub const ENTRY_MERGE: &str = "entry_merge";
pub const ENTRY_CLONE: &str = "entry_clone";
pub const ENTRY_HASH_OBJECT: &str = "entry_hash-object";
pub const ENTRY_CAT_FILE: &str = "entry_cat-file";
pub const ENTRY_FETCH: &str = "entry_fetch";
pub const ENTRY_REMOTE: &str = "entry_remote";
pub const ENTRY_LS: &str = "entry_ls";
pub const ENTRY_TAG: &str = "entry_tag";
pub const ENTRY_CHECK_IGNORE: &str = "entry_ignore";
pub const ENTRY_REBASE: &str = "entry_rebase";
pub const ENTRY_PULL: &str = "entry_pull";
pub const ENTRY_PUSH: &str = "entry_push";

pub fn get_entries() -> Vec<String> {
    let entries: Vec<String> = vec![
        ENTRY_CONSOLE.to_string(),
        ENTRY_BRANCH.to_string(),
        ENTRY_CHECKOUT.to_string(),
        ENTRY_ADD_RM.to_string(),
        ENTRY_COMMIT.to_string(),
        ENTRY_MERGE.to_string(),
        ENTRY_CLONE.to_string(),
        ENTRY_HASH_OBJECT.to_string(),
        ENTRY_CAT_FILE.to_string(),
        ENTRY_FETCH.to_string(),
        ENTRY_REMOTE.to_string(),
        ENTRY_LS.to_string(),
        ENTRY_TAG.to_string(),
        ENTRY_REBASE.to_string(),
        ENTRY_CHECK_IGNORE.to_string(),
        ENTRY_PUSH.to_string(),
        ENTRY_PULL.to_string(),
    ];
    entries
}
