use std::net::TcpStream;
use messages::buf::{HEADER_LEN, MAX_MSG_SIZE};

pub struct Connection {
    pub inner: TcpStream,
    pub state: ConnectionState,
    pub read_buf_size: usize,
    pub read_buf: [u8; HEADER_LEN + MAX_MSG_SIZE],
    pub write_buf_size: usize,
    pub write_buf_sent: usize,
    pub write_buf: [u8; HEADER_LEN + MAX_MSG_SIZE],
}

#[derive(PartialEq, Eq)]
pub enum ConnectionState {
    Reading,
    Responding,
    End,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Connection {
            inner: stream,
            state: ConnectionState::Reading,
            read_buf: [0u8; HEADER_LEN + MAX_MSG_SIZE],
            read_buf_size: 0,
            write_buf: [0u8; HEADER_LEN + MAX_MSG_SIZE],
            write_buf_size: 0,
            write_buf_sent: 0,
        }
    }
}
