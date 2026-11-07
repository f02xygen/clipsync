use std::{
    hash::{DefaultHasher, Hasher},
    io::{Read, Write},
    net::TcpStream,
};

pub struct Message {
    pub sender: String,
    pub body: String,
}

fn bytes_to_string(bytes: Vec<u8>) -> std::io::Result<String> {
    String::from_utf8(bytes).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

pub fn read_message(stream: &mut TcpStream) -> std::io::Result<Message> {
    let mut len_sender_bytes = [0u8; 4];
    stream.read_exact(&mut len_sender_bytes)?;
    let len_sender = u32::from_be_bytes(len_sender_bytes) as usize;
    let mut sender = vec![0u8; len_sender];
    stream.read_exact(&mut sender)?;

    let mut len_body_bytes = [0u8; 4];
    stream.read_exact(&mut len_body_bytes)?;
    let len_body = u32::from_be_bytes(len_body_bytes) as usize;
    let mut body = vec![0u8; len_body];
    stream.read_exact(&mut body)?;

    let sender = bytes_to_string(sender)?;
    let body = bytes_to_string(body)?;

    Ok(Message { sender, body })
}

pub fn send_message(stream: &mut TcpStream, msg: &Message) -> std::io::Result<()> {
    let len_sender_bytes = (msg.sender.len() as u32).to_be_bytes();
    stream.write_all(&len_sender_bytes)?;
    stream.write_all(msg.sender.as_bytes())?;

    let len_body_bytes = (msg.body.len() as u32).to_be_bytes();
    stream.write_all(&len_body_bytes)?;
    stream.write_all(msg.body.as_bytes())?;

    stream.flush()?;
    Ok(())
}

pub fn compute_hash(data: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    hasher.write(data);
    hasher.finish()
}
