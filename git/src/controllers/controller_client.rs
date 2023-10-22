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
                if let Err(error) = stream.write(command_bytes) {
                    eprintln!("Error al escribir: {}", error);
                }

                stream
                    .shutdown(std::net::Shutdown::Both)
                    .expect("Falló al cerrar la conexion");
            }
            Err(e) => {
                println!("{}", e.message());
            }
        };

        Ok(())
    }
}
