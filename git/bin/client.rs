use git::commands::clone::git_clone;
use git::config::Config;
use git::util::connections::start_client;

fn main() {
    let config = match Config::new() {
        Ok(config) => config,
        Err(error) => {
            println!("Error: {}", error.message());
            return;
        }
    };
    print!("{}", config);

    let addres = format!("{}:{}", config.ip, config.port);
    println!("Me voy a conectar a: {}", addres);

    let mut socket = match start_client(&addres) {
        Ok(s) => s,
        Err(e) => {
            println!("{}", e.message());
            return;
        }
    };

    println!("Conexión establecida con el servidor");
    git_clone(&mut socket).unwrap();
    // let repo_name = "sisop_2023a_ricaldi";
    // git_clone(&mut socket, repo_name).unwrap();
}
