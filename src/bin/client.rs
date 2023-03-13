use std::str;
use std::{
    net::TcpStream,
};

#[path = "../buf.rs"]
mod buf;
use crate::buf::{read_full, write_all, HEADER_LEN, MAX_MSG_SIZE};

fn query(connection: &mut TcpStream, text: &str) -> Result<(), std::io::Error> {
    let len = text.len();
    if len > MAX_MSG_SIZE {
        // TODO: Don't use this
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Message too long",
        ));
    }

    let mut write_buf = [0u8; HEADER_LEN + MAX_MSG_SIZE];
    // XXX: I am just going to do as although it is not the smart thing to do
    write_buf[..4].copy_from_slice(&(len as u32).to_le_bytes());
    write_buf[4..(4 + len)].copy_from_slice(text.as_bytes());
    write_all(connection, &mut write_buf, HEADER_LEN + len)?;

    let mut read_buf = [0u8; HEADER_LEN + MAX_MSG_SIZE + 1];
    read_full(connection, &mut read_buf, HEADER_LEN)?;
    let read_len = u32::from_le_bytes(read_buf[..4].try_into().unwrap());

    if read_len > MAX_MSG_SIZE as u32 {
        // TODO: Don't use this
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Message too long",
        ));
    }

    read_full(connection, &mut read_buf[4..], read_len as usize)?;
    read_buf[HEADER_LEN + read_len as usize] = 0;
    println!("Server says: {}", str::from_utf8(&read_buf[4..]).unwrap());

    Ok(())
}

fn main() {
    println!("hello world, i'm a client");
    let mut stream = TcpStream::connect("127.0.0.1:1234").expect("failed to connect");

    query(&mut stream, "hello1").unwrap();
    query(&mut stream, "hello2").unwrap();
    query(&mut stream, "hello3").unwrap();
}
