use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::connections::start_client;
use std::io::Write;
use crate::commands::branch::handle_branch;
use crate::commands::clone::handle_clone;
use crate::commands::commit::handle_commit;
use crate::commands::init::handle_init;
use crate::commands::cat_file::handle_cat_file;
use crate::commands::add::handle_add;
use crate::commands::checkout::handle_checkout;
use crate::commands::fetch::handle_fetch;
use crate::commands::hash_object::handle_hash_object;

#[derive(Clone)]
pub struct Controller {
    client: Client,
}

impl Controller {
    pub fn new(client: Client) -> Controller {
        Controller { client }
    }
    pub fn send_command(&self, command: String) -> Result<(), GitError> {
        let cloned_client = self.client.clone();

        handle_command(command.clone(), self.client.clone())?;

        match start_client(&cloned_client.get_ip()) {
            Ok(mut stream) => {
                let command_bytes = command.trim().as_bytes();
                if let Err(_) = stream.write(command_bytes) {
                    return Err(GitError::WriteStreamError);
                }

                if let Err(_) = stream.shutdown(std::net::Shutdown::Both) {
                    return Err(GitError::ServerConnectionError);
                }
            }
            Err(_) => return Err(GitError::GtkFailedInitiliaze),
        };
        Ok(())
    }
}

/// Esta función se encarga de llamar a al comando adecuado con los parametros necesarios
/// ###Parametros:
/// 'buffer': String que contiene el comando que se le pasara al servidor
fn handle_command(buffer: String, client: Client) -> Result<(), GitError> {
    let command = buffer.trim();
    let commands = command.split_whitespace().collect::<Vec<&str>>();
    let rest_of_command = commands.iter().skip(2).cloned().collect::<Vec<&str>>();
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
            _ => {
                return Err(GitError::CommandNotRecognizedError);
            }
        }
    } else {
        return Err(GitError::CommandDoesNotStartWithGitError);
    }
    Ok(())
}
