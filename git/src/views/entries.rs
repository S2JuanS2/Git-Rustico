pub const ENTRY_CONSOLE: &str = "entry_console";
pub const ENTRY_BRANCH: &str = "entry_branch";
pub const ENTRY_CHECKOUT: &str = "entry_checkout";
pub const ENTRY_ADD_RM: &str = "entry_add_rm";
pub const ENTRY_COMMIT: &str = "entry_commit";
pub const ENTRY_MERGE: &str = "entry_merge";
pub const ENTRY_CLONE: &str = "entry_clone";
pub const ENTRY_HASH_OBJECT: &str = "entry_hash-object";
pub const ENTRY_CAT_FILE: &str = "entry_cat-file";

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
    ];
    entries
}
