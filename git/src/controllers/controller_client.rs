use crate::commands::add::handle_add;
use crate::commands::branch::handle_branch;
use crate::commands::cat_file::handle_cat_file;
use crate::commands::checkout::handle_checkout;
use crate::commands::clone::handle_clone;
use crate::commands::commit::handle_commit;
use crate::commands::fetch::handle_fetch;
use crate::commands::hash_object::handle_hash_object;
use crate::commands::init::handle_init;
use crate::commands::status::handle_status;
use crate::errors::GitError;
use crate::models::client::Client;

#[derive(Clone)]
pub struct Controller {
    client: Client,
}

impl Controller {
    pub fn new(client: Client) -> Controller {
        Controller { client }
    }
    pub fn send_command(&self, command: &str) -> Result<(), GitError> {
        handle_command(command.to_string().clone(), self.client.clone())
    }
}

/// Esta función se encarga de llamar a al comando adecuado con los parametros necesarios
/// ###Parametros:
/// 'buffer': String que contiene el comando que se le pasara al servidor
fn handle_command(buffer: String, client: Client) -> Result<(), GitError> {
    let command = buffer.trim();
    let commands = command.split_whitespace().collect::<Vec<&str>>();
    let rest_of_command = commands.iter().skip(2).cloned().collect::<Vec<&str>>();

    if command.is_empty(){
        return Err(GitError::CommandDoesNotStartWithGitError);
    }

    if command.split_whitespace().count() == 1{
        return Err(GitError::CommandDoesNotStartWithGitError);
    }

    if commands[0] == "git" {
        match commands[1] {
            "branch" => {
                handle_branch(rest_of_command, client)?;
            }
            "clone" => {
                handle_clone(rest_of_command, client)?;
            }
            "commit" => {
                handle_commit(rest_of_command, client)?;
            }
            "init" => {
                handle_init(rest_of_command, client)?;
            }
            "cat_file" => {
                handle_cat_file(rest_of_command, client)?;
            }
            "add" => {
                handle_add(rest_of_command, client)?;
            }
            "checkout" => {
                handle_checkout(rest_of_command, client)?;
            }
            "fetch" => {
                handle_fetch(rest_of_command, client)?;
            }
            "hash_object" => {
                handle_hash_object(rest_of_command)?;
            }
            "status" => {
                handle_status(rest_of_command, client)?;
            }
            _ => {
                return Err(GitError::CommandNotRecognizedError);
            }
        }
    } else {
        return Err(GitError::CommandDoesNotStartWithGitError);
    }
    Ok(())
}
