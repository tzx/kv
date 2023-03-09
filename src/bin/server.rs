use std::net::{TcpListener, TcpStream};

fn do_something(connection: TcpStream) {
}

fn main() {
    println!("hello i'm a server!");

    let listener = TcpListener::bind("127.0.0.1:1234").unwrap();
    for incoming in listener.incoming() {
        let stream = incoming.unwrap();
        do_something(stream);
    }
}
