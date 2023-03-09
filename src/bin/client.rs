use std::{net::TcpStream, io::{Write, Read}};
use std::str;

fn main() {
    println!("hello world, i'm a client");
    let mut stream = TcpStream::connect("127.0.0.1:1234").expect("failed to connect");

    let _bytes_written = stream.write("hello".as_bytes()).expect("Failed to write to server");
    let mut read_buffer = [0u8; 64];
    let _bytes_read = stream.read(&mut read_buffer).expect("Failed to read from server");
    println!("server says: {}", str::from_utf8(&read_buffer).unwrap());
}
