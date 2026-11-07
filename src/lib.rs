use std::{
    hash::{DefaultHasher, Hasher},
    io::{Read, Write},
    net::TcpStream,
};

pub enum Payload {
    Text(String),
    Image(Vec<u8>),
}

impl Payload {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Payload::Text(text) => text.as_bytes(),
            Payload::Image(bytes) => bytes.as_slice(),
        }
    }
}

pub struct Message {
    pub sender: String,
    pub payload: Payload,
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

    let mut type_byte = [0u8; 1];
    stream.read_exact(&mut type_byte)?;
    let mut len_data_bytes = [0u8; 4];
    stream.read_exact(&mut len_data_bytes)?;

    let len_data = u32::from_be_bytes(len_data_bytes) as usize;
    let mut data_bytes = vec![0u8; len_data];
    stream.read_exact(&mut data_bytes)?;

    let sender = bytes_to_string(sender)?;
    let payload = match type_byte[0] {
        1 => Payload::Text(bytes_to_string(data_bytes)?),
        2 => Payload::Image(data_bytes),
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unknown payload type",
            ));
        }
    };

    Ok(Message { sender, payload })
}

pub fn send_message(stream: &mut TcpStream, msg: &Message) -> std::io::Result<()> {
    let len_sender_bytes = (msg.sender.len() as u32).to_be_bytes();
    stream.write_all(&len_sender_bytes)?;
    stream.write_all(msg.sender.as_bytes())?;

    match &msg.payload {
        Payload::Text(text) => {
            let type_tag = vec![1u8; 1];
            stream.write_all(&type_tag)?;
            let len_bytes = (text.len() as u32).to_be_bytes();
            stream.write_all(&len_bytes)?;
            stream.write_all(text.as_bytes())?;
        }
        Payload::Image(bytes) => {
            let type_tag = vec![2u8; 1];
            stream.write_all(&type_tag)?;
            let len_bytes = (bytes.len() as u32).to_be_bytes();
            stream.write_all(&len_bytes)?;
            stream.write_all(&bytes)?;
        }
    };

    stream.flush()?;
    Ok(())
}

pub fn compute_hash(data: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    hasher.write(data);
    hasher.finish()
}
