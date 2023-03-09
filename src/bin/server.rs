use std::{net::{TcpListener, TcpStream}, io::{Read, Write}};
use std::str;

fn do_something(mut connection: TcpStream) {
    let mut read_buffer = [0u8; 64];
    let _n = connection.read(&mut read_buffer).expect("Failed to read");
    println!("Client says: {}", str::from_utf8(&read_buffer).unwrap());
    connection.write("world".as_bytes()).expect("Failed to write");
}

fn main() {
    println!("hello i'm a server!");
    let listener = TcpListener::bind("127.0.0.1:1234").expect("Failed to bind");
    for incoming in listener.incoming() {
        let stream = incoming.unwrap();
        do_something(stream);
    }
}
