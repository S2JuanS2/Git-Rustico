use git::config::Config;
use git::errors::GitError;
use git::util::connections::start_server;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::{env, thread};


fn send_message_log(
    tx: & Arc<Mutex<Sender<String>>>,
    message: &str,
    error: GitError,
) -> Result<(), GitError> {
    match tx.lock().unwrap().send(message.to_string())
    {
        Ok(_) => Ok(()),
        Err(_) => Err(error),
    }
}
fn log_message(tx: &Arc<Mutex<Sender<String>>>, message: &str) -> Result<(), GitError> {
    send_message_log(tx, message, GitError::GenericError)?;
    Ok(())
}

fn init_new_client(stream: &TcpStream, tx: &Arc<Mutex<Sender<String>>>) -> Result<String, GitError> {
    // Leer datos del cliente
    let client_description = match stream.peer_addr() {
        Ok(addr) => {
            let message = format!("Conexión establecida con {}\n", addr);
            log_message(tx, &message)?;
            format!("Client {}: ", addr)
        },
        Err(_) => {
            log_message(tx, "Cliente desconocido conectado\n")?;
            "Cliente desconocido: ".to_string()
        }
    };
    Ok(client_description)
}

fn handle_client(stream: &mut TcpStream, tx: Arc<Mutex<Sender<String>>>) -> Result<(), GitError>{
    let client_description = init_new_client(stream, &tx)?;
    let mut buffer = [0; 1024]; // Buffer de lectura

    while let Ok(bytes_read) = stream.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }

        // Trabajar con los datos leídos en el buffer
        let data = &buffer[..bytes_read];
        // format!("Datos recibidos: {:?}", data);
        println!("Datos recibidos: {:?}", data);

    }

    // println!("Recibido: {}", String::from_utf8_lossy(&buffer));
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
        let _ = write_log_file(&config.path_log, rx);
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

fn write_log_file(log_path: &str, rx: Receiver<String>) -> Result<(), std::io::Error> {
    // Intentamos abrir el archivo en modo append o crearlo si no existe.
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;

    // Creamos un bucle para recibir datos del canal y escribirlos en el archivo.
    for received_data in rx {
        writeln!(file, "{}", received_data)?;
    }

    // Cerramos el archivo al finalizar.
    file.sync_all()?;
    Ok(())
}
