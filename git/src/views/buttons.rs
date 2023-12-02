pub const BUTTON_CLEAR: &str = "button_clear";
pub const BUTTON_SEND: &str = "button_send";
pub const BUTTON_INIT: &str = "button_init";
pub const BUTTON_BRANCH: &str = "button_branch";
pub const BUTTON_STATUS: &str = "button_status";
pub const BUTTON_CAT_FILE: &str = "button_cat-file";
pub const BUTTON_CMD_CAT_FILE_P: &str = "button_cmd_cat-file_p";
pub const BUTTON_CMD_CAT_FILE_T: &str = "button_cmd_cat-file_t";
pub const BUTTON_CMD_CAT_FILE_S: &str = "button_cmd_cat-file_s";
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
pub const BUTTON_SHOW_REF: &str = "button_show_ref";
pub const BUTTON_LS_TREE: &str = "button_ls_tree";
pub const BUTTON_LS_FILES: &str = "button_ls_files";
pub const BUTTON_CHECK_IGNORE: &str = "button_check_ignore";
pub const BUTTON_TAG: &str = "button_tag";
pub const BUTTON_TAG_CREATE: &str = "button_tag_create";
pub const BUTTON_TAG_DELETE: &str = "button_tag_delete";
pub const BUTTON_REBASE: &str = "button_rebase";
pub const BUTTON_CMD_CLONE: &str = "button_cmd_clone";
pub const BUTTON_CMD_HASH_OBJECT: &str = "button_cmd_hash-object";
pub const BUTTON_CMD_FETCH: &str = "button_cmd_fetch";
pub const BUTTON_CMD_PUSH: &str = "button_cmd_push";
pub const BUTTON_CMD_PULL: &str = "button_cmd_pull";
pub const BUTTON_HELP: &str = "button_help";

pub fn get_buttons() -> Vec<String> {
    let buttons: Vec<String> = vec![
        BUTTON_CLEAR.to_string(),
        BUTTON_SEND.to_string(),
        BUTTON_INIT.to_string(),
        BUTTON_BRANCH.to_string(),
        BUTTON_STATUS.to_string(),
        BUTTON_CAT_FILE.to_string(),
        BUTTON_CMD_CAT_FILE_P.to_string(),
        BUTTON_CMD_CAT_FILE_S.to_string(),
        BUTTON_CMD_CAT_FILE_T.to_string(),
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
        BUTTON_SHOW_REF.to_string(),
        BUTTON_LS_TREE.to_string(),
        BUTTON_LS_FILES.to_string(),
        BUTTON_CHECK_IGNORE.to_string(),
        BUTTON_TAG.to_string(),
        BUTTON_TAG_CREATE.to_string(),
        BUTTON_TAG_DELETE.to_string(),
        BUTTON_REBASE.to_string(),
        BUTTON_CMD_CLONE.to_string(),
        BUTTON_CMD_HASH_OBJECT.to_string(),
        BUTTON_CMD_FETCH.to_string(),
        BUTTON_CMD_PUSH.to_string(),
        BUTTON_CMD_PULL.to_string(),
        BUTTON_HELP.to_string(),
    ];
    buttons
}
