use clipsync::{Message, Payload, compute_hash, read_message, send_message};
use std::{
    net::TcpStream,
    process::Command,
    process::Stdio,
    sync::{Arc, Mutex},
};

fn get_clipboard() -> Option<Payload> {
    let types = Command::new("wl-paste").arg("-l").output().ok()?;
    if !types.status.success() {
        return None;
    }
    let types_str = String::from_utf8_lossy(&types.stdout);

    if types_str.contains("image/png") {
        let output = Command::new("wl-paste")
            .arg("-t")
            .arg("image/png")
            .output()
            .ok()?;
        if output.status.success() && !output.stdout.is_empty() {
            Some(Payload::Image(output.stdout))
        } else {
            None
        }
    } else {
        let output = Command::new("wl-paste").arg("-n").output().ok()?;
        if output.status.success() {
            let text = String::from_utf8(output.stdout).ok()?;
            if !text.is_empty() {
                Some(Payload::Text(text))
            } else {
                None
            }
        } else {
            None
        }
    }
}

fn set_clipboard(payload: &Payload) {
    match payload {
        Payload::Text(text) => {
            let _ = Command::new("wl-copy")
                .arg("--")
                .arg(text)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        Payload::Image(bytes) => {
            use std::io::Write;
            if let Ok(mut child) = Command::new("wl-copy")
                .arg("-t")
                .arg("image/png")
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(bytes);
                }
                let _ = child.wait();
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let initial_hash = get_clipboard()
        .map(|p| compute_hash(p.as_bytes()))
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
                    match &msg.payload {
                        Payload::Text(text) => println!("Got from {} [text]: {}", msg.sender, text),
                        Payload::Image(bytes) => {
                            println!("Got from {} [image]: {} bytes", msg.sender, bytes.len())
                        }
                    }
                    let h = compute_hash(msg.payload.as_bytes());
                    *last_hash_clone.lock().unwrap() = h;
                    set_clipboard(&msg.payload);
                }
                Err(_) => {
                    println!("Disconnected from server.");
                    std::process::exit(1);
                }
            }
        }
    });

    loop {
        std::thread::sleep(std::time::Duration::from_millis(300));
        if let Some(payload) = get_clipboard() {
            let h = compute_hash(payload.as_bytes());
            let mut guard = last_hash.lock().unwrap();
            if *guard != h {
                match &payload {
                    Payload::Text(text) => {
                        println!(">> Found new text [{} bytes]: {:?}", text.len(), text);
                    }
                    Payload::Image(bytes) => {
                        println!(">> Found new image [{} bytes]", bytes.len());
                    }
                }

                *guard = h;
                send_message(
                    &mut stream,
                    &Message {
                        sender: client_name.clone(),
                        payload,
                    },
                )?;
                println!(">> Successfully sent to network!");
            }
        }
    }
}
