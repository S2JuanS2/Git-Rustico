use git::config::Config;
use git::controllers::controller_client::Controller;
use git::errors::GitError;
use git::models::client::Client;
use git::views::view_client::View;
use std::env;
use std::fs;

fn main() -> Result<(), GitError> {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(args)?;
    print!("{}", config);

    let address = format!("{}:{}", config.ip, config.port);

    let mut client = Client::new(
        config.name,
        config.email,
        address,
        config.src,
        config.path_log,
    );

    let current_src = "./";
    let files = match fs::read_dir(current_src) {
        Ok(files) => files,
        Err(_) => return Err(GitError::ReadDirError),
    };
    for file in files {
        let entry = match file {
            Ok(entry) => entry,
            Err(_) => return Err(GitError::DirEntryError),
        };
        if entry.file_type().map_or(false, |ft| ft.is_dir()) {
            if let Some(name) = entry.file_name().to_str() {
                let src_complete = format!("{}/{}", current_src, name);
                if fs::read_dir(&format!("{}/.git", src_complete)).is_ok() {
                    client.set_directory_path(name.to_string());
                }
            }
        }
    }

    let controller = Controller::new(client);

    let mut view = View::new(controller)?;

    view.start_view()?;

    Ok(())
}
