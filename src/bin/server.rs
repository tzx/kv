use std::{
    io,
    net::{TcpListener, TcpStream},
    str,
};

#[path = "../buf.rs"]
mod buf;
use crate::buf::{read_full, write_all, HEADER_LEN, MAX_MSG_SIZE};

fn one_request(connection: &mut TcpStream) -> Result<(), io::Error> {
    let mut read_buf = [0u8; HEADER_LEN + MAX_MSG_SIZE + 1];
    // TODO: read_buf needs to be slice of size 4, so it reads 4
    read_full(connection, &mut read_buf[..4], 4)?;

    // unwrap because otherwise we would have a read error above
    let len = u32::from_le_bytes(read_buf[..4].try_into().unwrap()) as usize;
    if len > MAX_MSG_SIZE {
        eprintln!("Message too long");
        // TODO: custom error, not really IO
        return Err(io::Error::new(io::ErrorKind::Other, "Message too long"));
    }

    read_full(connection, &mut read_buf[4..(4 + len)], len)?;
    read_buf[HEADER_LEN + len] = 0;
    println!("Client says: {}", str::from_utf8(&read_buf[4..]).unwrap());

    const RESPONSE: &str = "world";
    let response_len = RESPONSE.len();
    let mut write_buf = [0u8; HEADER_LEN + RESPONSE.as_bytes().len()];
    write_buf[..4].copy_from_slice(&(response_len as u32).to_le_bytes());
    write_buf[4..].copy_from_slice(RESPONSE.as_bytes());
    write_all(connection, &mut write_buf, HEADER_LEN + response_len)?;

    Ok(())
}

fn main() {
    println!("hello i'm a server!");
    let listener = TcpListener::bind("127.0.0.1:1234").expect("Failed to bind");
    for incoming in listener.incoming() {
        let mut stream = incoming.unwrap();

        // Only serve one client connection at a time
        loop {
            // TODO: match error and break
            one_request(&mut stream).unwrap();
        }
    }
}
