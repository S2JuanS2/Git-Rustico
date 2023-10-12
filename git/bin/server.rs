use git::config::Config;
use git::util::connections::start_server;


fn main() {
    let config = match Config::new() {
        Ok(config) => config,
        Err(error) => {
            println!("Error: {}", error.message());
            return;
        }
    };
    print!("{}", config);

    let address = format!("{}:{}", config.ip, config.port);
    println!("{}", address);

    // Escucha en la dirección IP y el puerto deseados
    let listener = match start_server(&address) {
        Ok(listener) => listener,
        Err(e) => {
            println!("{}", e.message());
            return;
        }
    };
    println!("Servidor escuchando en 127.0.0.1:8080");

    match listener.accept() {
        Ok((_socket, address)) => println!("Nueva conexión: {}", address),
        Err(e) => println!("Error: {}", e),
    }
}
