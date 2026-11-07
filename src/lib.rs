use std::{
    hash::{DefaultHasher, Hasher},
    io::{Read, Write},
    net::TcpStream,
};

pub fn read_message(stream: &mut TcpStream) -> std::io::Result<String> {
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes)?;
    let len = u32::from_be_bytes(len_bytes) as usize;

    let mut body = vec![0u8; len];
    stream.read_exact(&mut body)?;

    String::from_utf8(body).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

pub fn send_message(stream: &mut TcpStream, msg: &str) -> std::io::Result<()> {
    let len_bytes = (msg.len() as u32).to_be_bytes();
    stream.write_all(&len_bytes)?;
    stream.write_all(msg.as_bytes())?;
    stream.flush()?;

    Ok(())
}

pub fn compute_hash(data: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    hasher.write(data);
    hasher.finish()
}
