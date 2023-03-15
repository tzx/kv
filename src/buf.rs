use std::io::{self, Read, Write};
use std::net::TcpStream;

pub const MAX_MSG_SIZE: usize = 4096;
// Header is 4 bytes indicating length of message
pub const HEADER_LEN: usize = 4;

// TODO: It should just be read_exact instead of doing the while loop

pub fn read_full(
    connection: &mut TcpStream,
    read_buf: &mut [u8],
    mut n: usize,
) -> Result<(), io::Error> {
    let og_n = n;
    while n > 0 {
        let amt_read = connection.read(&mut read_buf[(og_n - n)..n])?;
        assert!(amt_read <= n);
        n -= amt_read;
    }
    Ok(())
}

pub fn write_all(
    connection: &mut TcpStream,
    buf: &mut [u8],
    mut n: usize,
) -> Result<(), io::Error> {
    let mut pos = 0;
    while n > 0 {
        let amt_write = connection.write(&buf[pos..(pos + n)])?;
        assert!(amt_write <= n);
        n -= amt_write;
        pos += amt_write;
    }
    Ok(())
}
