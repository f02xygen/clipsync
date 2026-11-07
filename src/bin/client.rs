use clipsync::{Message, compute_hash, read_message, send_message};
use std::{
    net::TcpStream,
    process::Command,
    process::Stdio,
    sync::{Arc, Mutex},
};

fn get_clipboard() -> Option<String> {
    let command = Command::new("wl-paste").arg("-n").output().ok()?;
    if command.status.success() {
        String::from_utf8(command.stdout).ok()
    } else {
        None
    }
}

fn set_clipboard(text: &str) {
    let _ = Command::new("wl-copy")
        .arg("--")
        .arg(text)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

fn main() -> std::io::Result<()> {
    let initial_hash = get_clipboard()
        .map(|text| compute_hash(text.as_bytes()))
        .unwrap_or(0);
    let last_hash = Arc::new(Mutex::new(initial_hash));
    let last_hash_clone = Arc::clone(&last_hash);

    let server_addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:9090".to_string());
    let client_name = std::env::args().nth(2).unwrap_or_else(|| {
        std::fs::read_to_string("/etc/hostname")
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string())
    });
    let mut stream = TcpStream::connect(&server_addr)?;
    println!("Client '{}' connected to {}!", client_name, server_addr);

    let mut reader = stream.try_clone()?;

    std::thread::spawn(move || {
        loop {
            match read_message(&mut reader) {
                Ok(msg) => {
                    println!("Got from {}: {}", msg.sender, msg.body);
                    let h = compute_hash(msg.body.as_bytes());
                    *last_hash_clone.lock().unwrap() = h;
                    set_clipboard(&msg.body);
                }
                Err(_) => {
                    println!("Disconnected from server.");
                    break;
                }
            }
        }
    });

    loop {
        std::thread::sleep(std::time::Duration::from_millis(300));
        if let Some(text) = get_clipboard() {
            if text.is_empty() {
                continue;
            }
            let h = compute_hash(text.as_bytes());
            let mut guard = last_hash.lock().unwrap();
            if *guard != h {
                println!(">> Found new buffer [{} bytes]: {:?}", text.len(), text);
                *guard = h;
                send_message(
                    &mut stream,
                    &Message {
                        sender: client_name.clone(),
                        body: text,
                    },
                )?;
                println!(">> Successfully sent to network!");
            }
        }
    }

    Ok(())
}
