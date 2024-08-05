use git::config::Config;
use git::controllers::controller_client::Controller;
use git::errors::GitError;
use git::models::client::Client;
// use git::util::files::is_git_initialized;
use git::views::view_client::View;
use std::env;

fn main() -> Result<(), GitError> {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(args)?;
    print!("{}", config);

    let address = format!("{}:{}", config.ip, config.port_daemon);

    let client = Client::new(
        config.name,
        config.email,
        config.ip,
        config.port_daemon,
        address,
        config.src,
        config.path_log,
    );

    // let init = is_git_initialized(client.get_directory_path())?;
    // if init.0 {
    //     client.set_directory_path(init.1);
    // }

    let controller = Controller::new(client);

    let mut view = View::new(controller)?;

    view.start_view()?;

    Ok(())
}
