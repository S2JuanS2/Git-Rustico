use crate::commands::add::handle_add;
use crate::commands::branch::handle_branch;
use crate::commands::cat_file::handle_cat_file;
use crate::commands::checkout::handle_checkout;
use crate::commands::clone::handle_clone;
use crate::commands::commit::handle_commit;
use crate::commands::fetch::handle_fetch;
use crate::commands::hash_object::handle_hash_object;
use crate::commands::init::handle_init;
use crate::commands::log::handle_log;
use crate::commands::merge::handle_merge;
use crate::commands::pull::handle_pull;
use crate::commands::push::handle_push;
use crate::commands::remote::handle_remote;
use crate::commands::rm::handle_rm;
use crate::commands::status::handle_status;

use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::logger::write_client_log;

#[derive(Clone)]
pub struct Controller {
    client: Client,
}

impl Controller {
    pub fn new(client: Client) -> Controller {
        Controller { client }
    }
    pub fn send_command(&mut self, command: &str) -> Result<String, GitError> {
        write_client_log(self.client.get_directory_path(), command.to_string(), self.client.get_path_log())?;
        match handle_command(command.to_string().clone(), &mut self.client) {
            Ok(result) => {
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
                    e.message().to_string(),
                    self.client.get_path_log(),
                )?;
                Err(e)
            }
        }
    }
    pub fn get_name_client(& self) -> &str{
        self.client.get_name()
    }
}

/// Esta función se encarga de llamar a al comando adecuado con los parametros necesarios
/// ###Parametros:
/// 'buffer': String que contiene el comando que se le pasara al servidor
fn handle_command(buffer: String, client: &mut Client) -> Result<String, GitError> {
    let command = buffer.trim();
    let commands = command.split_whitespace().collect::<Vec<&str>>();
    let rest_of_command = commands.iter().skip(2).cloned().collect::<Vec<&str>>();

    let mut result = " ".to_string();

    if command.is_empty() {
        return Err(GitError::NonGitCommandError);
    }

    if command.split_whitespace().count() == 1 {
        return Err(GitError::NonGitCommandError);
    }

    if commands[0] == "git" {
        match commands[1] {
            "branch" => {
                result = handle_branch(rest_of_command, client.clone())?;
            }
            "clone" => {
                if let Some(path_clone) = rest_of_command.get(0){
                    client.set_directory_path(path_clone.to_string());
                }
                result = handle_clone(rest_of_command, client.clone())?;
            }
            "commit" => {
                result = handle_commit(rest_of_command, client.clone())?;
            }
            "init" => {
                result = handle_init(rest_of_command, client.clone())?;
            }
            "cat-file" => {
                result = handle_cat_file(rest_of_command, client.clone())?;
            }
            "add" => {
                result = handle_add(rest_of_command, client.clone())?;
            }
            "checkout" => {
                result = handle_checkout(rest_of_command, client.clone())?;
            }
            "fetch" => {
                handle_fetch(rest_of_command, client.clone())?;
            }
            "hash-object" => {
                result = handle_hash_object(rest_of_command, client.clone())?;
            }
            "status" => {
                result = handle_status(rest_of_command, client.clone())?;
            }
            "log" => {
                result = handle_log(rest_of_command, client.clone())?;
            }
            "pull" => {
                handle_pull(rest_of_command, client.clone())?;
            }
            "push" => {
                handle_push(rest_of_command, client.clone())?;
            }
            "merge" => {
                result = handle_merge(rest_of_command, client.clone())?;
            }
            "remote" => {
                handle_remote(rest_of_command, client.clone())?;
            }
            "rm" => {
                result = handle_rm(rest_of_command, client.clone())?;
            }
            _ => {
                return Err(GitError::CommandNotRecognizedError);
            }
        }
    } else {
        return Err(GitError::NonGitCommandError);
    }
    Ok(result)
}
