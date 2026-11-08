use clipsync::{read_message, send_message};
use std::{
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

fn handle_client(mut stream: TcpStream, addr: SocketAddr, clients: Arc<Mutex<Vec<TcpStream>>>) {
    if let Ok(addr) = stream.peer_addr() {
        println!("Client connected: {}", addr);
    }
    loop {
        match read_message(&mut stream) {
            Err(e) if e.kind() == std::io::ErrorKind::InvalidData => println!("Invalid UTF-8 data"),
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            Ok(msg) => {
                println!("Got from {}", msg.sender);
                let mut clients_guard = clients.lock().unwrap();
                for other_client in clients_guard.iter_mut() {
                    if other_client.peer_addr().ok() != Some(addr) {
                        if let Err(e) = send_message(other_client, &msg) {
                            println!("Error: {}", e);
                            continue;
                        }
                    }
                }
            }
        }
    }

    clients
        .lock()
        .unwrap()
        .retain(|c| c.peer_addr().ok() != Some(addr));
}

fn main() -> std::io::Result<()> {
    let listener_addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:9090".to_string());
    let listener = TcpListener::bind(&listener_addr)?;
    let clients = Arc::new(Mutex::new(Vec::<TcpStream>::new()));
    println!("Server started!");
    for stream in listener.incoming() {
        let stream = stream?;
        let addr = stream.peer_addr()?;
        let writer = stream.try_clone()?;
        let mut clients_guard = clients.lock().unwrap();
        clients_guard.push(writer);
        let clients_clone = Arc::clone(&clients);
        std::thread::spawn(move || {
            handle_client(stream, addr, clients_clone);
        });
    }

    Ok(())
}
