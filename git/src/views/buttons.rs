pub const BUTTON_CLEAR: &str = "button_clear";
pub const BUTTON_SEND: &str = "button_send";
pub const BUTTON_INIT: &str = "button_init";
pub const BUTTON_BRANCH: &str = "button_branch";
pub const BUTTON_STATUS: &str = "button_status";
pub const BUTTON_CAT_FILE: &str = "button_cat-file";
pub const BUTTON_PULL: &str = "button_pull";
pub const BUTTON_PUSH: &str = "button_push";
pub const BUTTON_FETCH: &str = "button_fetch";
pub const BUTTON_REMOTE: &str = "button_remote";
pub const BUTTON_LOG: &str = "button_log";
pub const BUTTON_HASH_OBJECT: &str = "button_hash-object";
pub const BUTTON_ADD: &str = "button_add";
pub const BUTTON_RM: &str = "button_rm";
pub const BUTTON_CHECKOUT: &str = "button_checkout";
pub const BUTTON_COMMIT: &str = "button_commit";
pub const BUTTON_MERGE: &str = "button_merge";
pub const BUTTON_CLONE: &str = "button_clone";
pub const BUTTON_CMD_CLONE: &str = "button_cmd_clone";
pub const BUTTON_CMD_HASH_OBJECT: &str = "button_cmd_hash-object";
pub const BUTTON_CMD_CAT_FILE: &str = "button_cmd_cat-file";

pub fn get_buttons() -> Vec<String> {
    let buttons: Vec<String> = vec![
        BUTTON_CLEAR.to_string(),
        BUTTON_SEND.to_string(),
        BUTTON_INIT.to_string(),
        BUTTON_BRANCH.to_string(),
        BUTTON_STATUS.to_string(),
        BUTTON_CAT_FILE.to_string(),
        BUTTON_PULL.to_string(),
        BUTTON_PUSH.to_string(),
        BUTTON_FETCH.to_string(),
        BUTTON_REMOTE.to_string(),
        BUTTON_LOG.to_string(),
        BUTTON_HASH_OBJECT.to_string(),
        BUTTON_ADD.to_string(),
        BUTTON_RM.to_string(),
        BUTTON_CHECKOUT.to_string(),
        BUTTON_COMMIT.to_string(),
        BUTTON_MERGE.to_string(),
        BUTTON_CLONE.to_string(),
        BUTTON_CMD_CLONE.to_string(),
        BUTTON_CMD_HASH_OBJECT.to_string(),
        BUTTON_CMD_CAT_FILE.to_string(),
    ];
    buttons
}