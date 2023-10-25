use git::config::Config;
use git::errors::GitError;
use git::util::connections::start_server;
use std::fs::OpenOptions;
use std::sync::{Mutex, Arc};
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread::JoinHandle;
use std::{env, thread};
use std::io::Read;
use std::net::{TcpStream, TcpListener};
use std::io::Write;

fn handle_client(mut stream: TcpStream, tx: Arc<Mutex<Sender<String>>>) {
    // Leer datos del cliente
    let client_description = stream.peer_addr().ok().map_or_else(
        || "Cliente desconocido conectado".to_string(),
        |peer_addr| format!("Conexión establecida con {}", peer_addr),
    );
    
    {
        let _ = &tx.lock().unwrap().send(client_description).unwrap();
    }

    let mut buffer = [0; 1024];

    if let Err(error) = stream.read(&mut buffer) {
        eprintln!("Error al leer: {}", error);
        return;
    }
    println!("Recibido: {}", String::from_utf8_lossy(&buffer));
}

fn main() -> Result<(), GitError> {
    let args: Vec<String> = env::args().collect();
    let config =  Config::new(args)?;
    print!("{}", config);

    let address = format!("{}:{}", config.ip, config.port);

    // Escucha en la dirección IP y el puerto deseados
    let listener = start_server(&address)?;

    let (tx, rx) = mpsc::channel();

    let log = thread::spawn(move || {
        let _ = write_log_file(&config.path_log, rx);
    });

    let clients = thread::spawn(move || {
        let _ = receive_client(&listener, tx);
    });

    clients.join().expect("No hay clientes");
    log.join().expect("No se pudo escribir el archivo de log");

    Ok(())
}

fn receive_client(listener: & TcpListener, tx: Sender<String>) -> Result<Vec<JoinHandle<()>>, GitError> {
    let shared_tx = Arc::new(Mutex::new(tx));
    let mut handles: Vec<JoinHandle<()>> = vec![];
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let tx = Arc::clone(&shared_tx);
                println!("Nueva conexión: {:?}", stream.local_addr());
                handles.push(std::thread::spawn(|| {
                    handle_client(stream, tx);
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