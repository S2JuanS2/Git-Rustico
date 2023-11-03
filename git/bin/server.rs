use git::config::Config;
use git::errors::GitError;
use git::util::connections::start_server;
use git::util::logger::{
    get_client_signature, handle_log_file, log_client_connect,
    log_message, log_client_disconnection_error, log_client_disconnection_success,
};
use git::util::git_request::GitRequest;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::{env, thread};


fn receive_request(stream: &mut TcpStream, signature: String, tx: Arc<Mutex<Sender<String>>>) -> Result<GitRequest, GitError>
{
    let request = GitRequest::read_git_request(stream);
    match request {
        Ok(request) => {
            let message = format!("{}{:?}", signature, request);
            log_message(&tx, &message);
            Ok(request)
        }
        Err(e) => {
            let message = format!("{}Error al procesar la petición: {}", signature, e);
            log_message(&tx, &message);
            log_client_disconnection_error(&tx, &signature);
            Err(e.into())
        }
    }
}

// fn process_request(stream: &mut TcpStream, tx: &Arc<Mutex<Sender<String>>>, signature: &String, request: &GitRequest) -> Result<(), GitError>
// {
//     match request.execute()
//     {
//         Ok(()) => {
//             let message = format!("{}Request exitosa", signature);
//             log_message(tx, &message);
//             Ok(())
//         }
//         Err(e) => {
//             let message: String = format!("{}Error al procesar la petición: {}", signature, e);
//             log_message(tx, &message);
//             log_client_disconnection_error(tx, signature);
//             Err(e.into())
//         }
//     }
// }

fn handle_client(stream: &mut TcpStream, tx: Arc<Mutex<Sender<String>>>) -> Result<(), GitError> {
    log_client_connect(stream, &tx);
    let signature = get_client_signature(stream)?;

    let request = receive_request(stream, signature.clone(), tx.clone())?;

    // process_request(stream, &tx, &signature, &request)?;

    log_client_disconnection_success(&tx, &signature);
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
