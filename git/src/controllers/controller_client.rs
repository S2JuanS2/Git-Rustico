use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::connections::start_client;
use std::io::Write;

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
