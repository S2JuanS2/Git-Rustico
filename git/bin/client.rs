use git::commands::clone::git_clone;
use git::config::Config;
use std::env;
use git::models::client::Client;
use git::controllers::controller_client::Controller;
use git::views::view_client::View;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = match Config::new(args) {
        Ok(config) => config,
        Err(error) => {
            println!("Error: {}", error.message());
            return;
        }
    };
    print!("{}", config);

    let address = format!("{}:{}", config.ip, config.port);

    //Cambiar el directorio del cliente
    let client = Client::new(address, "./test/".to_string());

    let controller = Controller::new(client.clone());

    let view = View::new(controller.clone());
    
    match view.start_view(){
        Ok(_) => (),
        Err(error) => eprintln!("Error: {}", error.message()),
    }
}
