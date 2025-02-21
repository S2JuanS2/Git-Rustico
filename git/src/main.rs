use std::env;

use git::config::Config;

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
}
