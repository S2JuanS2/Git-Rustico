use git::config::Config;
use git::controllers::controller_client::Controller;
use git::errors::GitError;
use git::models::client::Client;
use git::views::view_client::View;
use std::env;

fn main() -> Result<(), GitError> {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(args)?;
    print!("{}", config);

    let address = format!("{}:{}", config.ip, config.port);

    let client = Client::new(address, "./test_repo".to_string());

    let controller = Controller::new(client.clone());

    let view = View::new(controller.clone())?;

    view.start_view()?;

    Ok(())
}
