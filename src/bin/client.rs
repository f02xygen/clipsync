use clipsync::{compute_hash, read_message, send_message};
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
    let mut stream = TcpStream::connect("127.0.0.1:9090")?;
    println!("Client connected!");

    let mut reader = stream.try_clone()?;

    std::thread::spawn(move || {
        loop {
            match read_message(&mut reader) {
                Ok(msg) => {
                    println!("Got: {}", msg);
                    let h = compute_hash(msg.as_bytes());
                    *last_hash_clone.lock().unwrap() = h;
                    set_clipboard(&msg);
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
            let h = compute_hash(text.as_bytes());
            let mut guard = last_hash.lock().unwrap();
            if *guard != h {
                println!(">> Found new bugger [{} bytes]: {:?}", text.len(), text);
                *guard = h;
                send_message(&mut stream, &text)?;
                println!(">> Successfully sent to network!");
            }
        }
    }

    Ok(())
}
