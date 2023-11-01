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
    pub fn send_command(&self, command: &str) -> Result<String, GitError> {
        let result = match handle_command(command.to_string().clone(), self.client.clone()) {
            Ok(result) => result,
            Err(e) => {
                write_client_log(self.client.get_directory_path(), e.message().to_string())?;
                return Err(e);
            }
        };
        Ok(result)
    }
}

/// Esta función se encarga de llamar a al comando adecuado con los parametros necesarios
/// ###Parametros:
/// 'buffer': String que contiene el comando que se le pasara al servidor
fn handle_command(buffer: String, client: Client) -> Result<String, GitError> {
    let command = buffer.trim();
    let directory = client.get_directory_path().to_string();
    let commands = command.split_whitespace().collect::<Vec<&str>>();
    let rest_of_command = commands.iter().skip(2).cloned().collect::<Vec<&str>>();

    let mut result = " ".to_string();

    if command.is_empty() {
        return Err(GitError::NonGitCommandError);
    }

    write_client_log(client.get_directory_path(), command.to_string())?;

    if command.split_whitespace().count() == 1 {
        return Err(GitError::NonGitCommandError);
    }

    if commands[0] == "git" {
        match commands[1] {
            "branch" => {
                result = handle_branch(rest_of_command, client)?;
            }
            "clone" => {
                let result = handle_clone(rest_of_command, client);
                println!("Result: {:?}", result);
                //return result;
            }
            "commit" => {
                handle_commit(rest_of_command, client)?;
            }
            "init" => {
                result = handle_init(rest_of_command, client)?;
            }
            "cat-file" => {
                result = handle_cat_file(rest_of_command, client)?;
            }
            "add" => {
                handle_add(rest_of_command, client)?;
            }
            "checkout" => {
                result = handle_checkout(rest_of_command, client)?;
            }
            "fetch" => {
                handle_fetch(rest_of_command, client)?;
            }
            "hash-object" => {
                result = handle_hash_object(rest_of_command, client)?;
            }
            "status" => {
                result = handle_status(rest_of_command, client)?;
            }
            "log" => {
                handle_log(rest_of_command, client)?;
            }
            "pull" => {
                handle_pull(rest_of_command, client)?;
            }
            "push" => {
                handle_push(rest_of_command, client)?;
            }
            "merge" => {
                handle_merge(rest_of_command, client)?;
            }
            "remote" => {
                handle_remote(rest_of_command, client)?;
            }
            "rm" => {
                handle_rm(rest_of_command, client)?;
            }
            _ => {
                return Err(GitError::CommandNotRecognizedError);
            }
        }
    } else {
        return Err(GitError::NonGitCommandError);
    }
    write_client_log(&directory, result.clone())?;
    Ok(result)
}
