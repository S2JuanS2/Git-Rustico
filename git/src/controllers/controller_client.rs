use crate::commands::{
    add::handle_add, branch::handle_branch, cat_file::handle_cat_file,
    check_ignore::handle_check_ignore, checkout::handle_checkout, clone::handle_clone,
    commit::handle_commit, errors::CommandsError, fetch::handle_fetch,
    hash_object::handle_hash_object, init::handle_init, log::handle_log, ls_files::handle_ls_files,
    ls_tree::handle_ls_tree, merge::handle_merge, pull::handle_pull, push::handle_push,
    rebase::handle_rebase, remote::handle_remote, rm::handle_rm, show_ref::handle_show_ref,
    status::handle_status, tag::handle_tag,
};

use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::files::is_git_initialized;
use crate::util::logger::write_client_log;

use crate::commands::branch::{get_current_branch, git_branch_list_display};
use gtk::prelude::LabelExt;

#[derive(Clone)]
pub struct Controller {
    client: Client,
    current_branch: String,
}

impl Controller {
    pub fn new(client: Client) -> Controller {
        let current_branch = "None".to_string();
        Controller {
            client,
            current_branch,
        }
    }
    pub fn send_command(&mut self, command: &str) -> Result<String, GitError> {
        match handle_command(command.to_string().clone(), &mut self.client) {
            Ok(result) => {
                write_client_log(
                    self.client.get_directory_path(),
                    command.to_string(),
                    self.client.get_path_log(),
                )?;
                write_client_log(
                    self.client.get_directory_path(),
                    "Successfully".to_string(),
                    self.client.get_path_log(),
                )?;
                Ok(result)
            }
            Err(e) => {
                write_client_log(
                    self.client.get_directory_path(),
                    command.to_string(),
                    self.client.get_path_log(),
                )?;
                write_client_log(
                    self.client.get_directory_path(),
                    e.message().to_string(),
                    self.client.get_path_log(),
                )?;
                Err(e)
            }
        }
    }
    pub fn get_name_client(&self) -> &str {
        self.client.get_name()
    }
    pub fn get_mail_client(&self) -> &str {
        self.client.get_email()
    }
    pub fn get_path_client(&self) -> &str {
        self.client.get_directory_path()
    }
    pub fn get_current_branch(&self) -> &str {
        &self.current_branch
    }
    pub fn set_current_branch(&mut self) -> Result<(), CommandsError> {
        let current_branch = get_current_branch(self.client.get_directory_path())?;
        self.current_branch = current_branch;
        Ok(())
    }
    pub fn set_branch_list(&self, label_branches: &gtk::Label) {
        let branches = match git_branch_list_display(self.client.get_directory_path()) {
            Ok(branches) => branches,
            Err(_) => return,
        };
        label_branches.set_text(&branches);
    }
    pub fn set_label_branch(&self, label_branch: &gtk::Label) {
        let current_branch = self.get_current_branch();
        label_branch.set_text(current_branch);
    }
    pub fn set_label_path(&self, label_path: &gtk::Label) {
        let path = self.get_path_client();
        label_path.set_text(path);
    }
}

/// Esta función se encarga de llamar a al comando adecuado con los parametros necesarios
/// ###Parametros:
/// 'buffer': String que contiene el comando que se le pasara al servidor
fn handle_command(buffer: String, client: &mut Client) -> Result<String, GitError> {
    let command = buffer.trim();
    let commands = command.split_whitespace().collect::<Vec<&str>>();
    let rest_of_command = commands.iter().skip(2).cloned().collect::<Vec<&str>>();

    let result;

    if command.is_empty() || command.split_whitespace().count() == 1 {
        return Err(GitError::NonGitCommandError);
    }
    let init = is_git_initialized(client.get_directory_path());
    if !init.0 && commands[1] != "init" && commands[1] != "clone" {
        return Err(GitError::NotAGitRepository);
    }

    if commands[0] == "git" {
        match commands[1] {
            "clone" => {
                let (status, new_path) = handle_clone(rest_of_command, client.clone())?;
                client.set_directory_path(new_path);
                result = status;
            }
            "fetch" => result = handle_fetch(rest_of_command, client.clone())?.to_string(),
            "branch" => result = handle_branch(rest_of_command, client.clone())?,
            "commit" => result = handle_commit(rest_of_command, client.clone())?,
            "init" => result = handle_init(rest_of_command, client.clone())?,
            "cat-file" => result = handle_cat_file(rest_of_command, client.clone())?,
            "add" => result = handle_add(rest_of_command, client.clone())?,
            "checkout" => result = handle_checkout(rest_of_command, client.clone())?,
            "hash-object" => result = handle_hash_object(rest_of_command, client.clone())?,
            "status" => result = handle_status(rest_of_command, client.clone())?,
            "log" => result = handle_log(rest_of_command, client.clone())?,
            "pull" => result = handle_pull(rest_of_command, client.clone())?,
            "push" => result = handle_push(rest_of_command, client.clone())?,
            "merge" => result = handle_merge(rest_of_command, client.clone())?,
            "remote" => result = handle_remote(rest_of_command, client.clone())?,
            "rm" => result = handle_rm(rest_of_command, client.clone())?,
            "ls-files" => result = handle_ls_files(rest_of_command, client.clone())?,
            "ls-tree" => result = handle_ls_tree(rest_of_command, client.clone())?,
            "check-ignore" => result = handle_check_ignore(rest_of_command, client.clone())?,
            "show-ref" => result = handle_show_ref(rest_of_command, client.clone())?,
            "tag" => result = handle_tag(rest_of_command, client.clone())?,
            "rebase" => result = handle_rebase(rest_of_command, client.clone())?,
            _ => return Err(GitError::CommandNotRecognizedError),
        }
    } else {
        return Err(GitError::NonGitCommandError);
    }
    Ok(result)
}
