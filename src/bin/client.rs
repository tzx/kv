use std::net::TcpStream;
use std::str;

#[path = "../buf.rs"]
mod buf;
use crate::buf::{read_full, write_all, HEADER_LEN, MAX_MSG_SIZE};

fn send_request(connection: &mut TcpStream, cmd: Vec<String>) -> Result<(), std::io::Error> {
    const LEN_SIZE: usize = 4;
    // nstr
    let mut len = LEN_SIZE;
    for s in cmd.iter() {
        // lenstr
        len += LEN_SIZE + s.len();
    }
    if len > MAX_MSG_SIZE {
        // TODO: Don't use this
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Message too long",
        ));
    }
    let mut write_buf = [0u8; HEADER_LEN + MAX_MSG_SIZE];
    write_buf[..HEADER_LEN].copy_from_slice(&(len as u32).to_le_bytes());
    let nstr = cmd.len();
    write_buf[HEADER_LEN..HEADER_LEN + LEN_SIZE].copy_from_slice(&(nstr as u32).to_le_bytes());

    let mut cur = HEADER_LEN + LEN_SIZE;
    for s in cmd.iter() {
        let slen = s.len();
        write_buf[cur..cur + LEN_SIZE].copy_from_slice(&(slen as u32).to_le_bytes());
        write_buf[cur + LEN_SIZE..cur + LEN_SIZE + slen].copy_from_slice(s.as_bytes());
        cur += LEN_SIZE + s.len();
    }
    write_all(connection, &mut write_buf, HEADER_LEN + len)?;
    Ok(())
}

fn read_response(connection: &mut TcpStream) -> Result<(), std::io::Error> {
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

    const CODE_SIZE: usize = 4;
    let res_code = u32::from_le_bytes((read_buf[HEADER_LEN..HEADER_LEN + CODE_SIZE]).try_into().unwrap());
    read_buf[HEADER_LEN + read_len as usize] = 0;
    println!("Server says: [{}] {}", res_code, str::from_utf8(&read_buf[4..]).unwrap());

    Ok(())
}

fn main() {
    println!("hello world, i'm a client");
    let mut stream = TcpStream::connect("127.0.0.1:1234").expect("failed to connect");

    let cmd = std::env::args().skip(1);
    send_request(&mut stream, cmd.collect()).unwrap();

    read_response(&mut stream).unwrap();
}
