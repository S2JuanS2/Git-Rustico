use git::config::Config;
use git::errors::GitError;
use git::util::connections::start_server;
use git::util::logger::{
    get_client_signature, handle_log_file, log_client_connect, log_client_disconnection,
    log_message,
};
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::{env, thread};

fn handle_client(stream: &mut TcpStream, tx: Arc<Mutex<Sender<String>>>) -> Result<(), GitError> {
    log_client_connect(stream, &tx);
    let signature = get_client_signature(stream)?;

    let mut buffer = [0; 2048]; // Buffer de lectura

    while let Ok(bytes_read) = stream.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }

        let data = &buffer[..bytes_read];
        let message = format!("{}Datos recibidos: {:?}", signature, data);
        log_message(&tx, &message);
    }

    log_client_disconnection(&tx, &signature);
    Ok(())
}

fn main() -> Result<(), GitError> {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(args)?;
    print!("{}", config);

    let address = format!("{}:{}", config.ip, config.port);

    // Escucha en la dirección IP y el puerto deseados
    let listener = start_server(&address)?;

    let (tx, rx) = mpsc::channel();

    let log: JoinHandle<()> = thread::spawn(move || {
        let _ = handle_log_file(&config.path_log, rx);
    });

    let clients = thread::spawn(move || {
        let _ = receive_client(&listener, tx);
    });

    clients.join().expect("No hay clientes");
    log.join().expect("No se pudo escribir el archivo de log");

    Ok(())
}

fn receive_client(
    listener: &TcpListener,
    tx: Sender<String>,
) -> Result<Vec<JoinHandle<()>>, GitError> {
    let shared_tx = Arc::new(Mutex::new(tx));
    let mut handles: Vec<JoinHandle<()>> = vec![];
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let tx = Arc::clone(&shared_tx);
                println!("Nueva conexión: {:?}", stream.local_addr());
                handles.push(std::thread::spawn(move || {
                    let _ = handle_client(&mut stream, tx);
                }));
            }
            Err(e) => {
                eprintln!("Error al aceptar la conexión: {}", e);
            }
        }
    }
    Ok(handles)
}
