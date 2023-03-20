use std::{
    collections::{HashMap, VecDeque},
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    str,
};

#[path = "../buf.rs"]
mod buf;
use crate::{
    buf::{HEADER_LEN, MAX_MSG_SIZE},
    messages::{Command, ResponseCode},
};

#[path = "../messages.rs"]
mod messages;
use crate::messages::to_cmd;

type KV = HashMap<String, String>;

#[derive(PartialEq, Eq)]
enum ConnectionState {
    Reading,
    Responding,
    End,
}

struct Connection {
    inner: TcpStream,
    state: ConnectionState,
    read_buf_size: usize,
    read_buf: [u8; HEADER_LEN + MAX_MSG_SIZE],
    write_buf_size: usize,
    write_buf_sent: usize,
    write_buf: [u8; HEADER_LEN + MAX_MSG_SIZE],
}

fn new_connection(stream: TcpStream) -> Connection {
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

fn try_one_request(connection: &mut Connection, kv: &mut KV) -> bool {
    if connection.read_buf_size < 4 {
        // Not enough data to read size header -> retry
        return false;
    }
    // unwrap because otherwise we would have a read error above
    let len = u32::from_le_bytes(connection.read_buf[..4].try_into().unwrap()) as usize;
    if len > MAX_MSG_SIZE {
        eprintln!("Message too long");
        connection.state = ConnectionState::End;
        return false;
    }

    if HEADER_LEN + len > connection.read_buf_size {
        // Not enough data to read data -> retry
        return false;
    }

    println!(
        "Client says: {}",
        str::from_utf8(&connection.read_buf[HEADER_LEN..HEADER_LEN + len]).unwrap()
    );

    let (to_write, tw_len, code) =
        match to_cmd(&connection.read_buf[HEADER_LEN..HEADER_LEN + len], len) {
            Ok(_cmd) => match _cmd {
                Command::Get(cmd) => {
                    kv.get(cmd.key).map_or_else(
                        || ("", 0, ResponseCode::Nonexistent),
                        |v| (v.as_str(), v.len(), ResponseCode::Success),
                    )
                }
                Command::Set(cmd) => {
                    kv.insert(cmd.key.to_string(), cmd.value.to_string());
                    ("", 0, ResponseCode::Success)
                }
                Command::Del(cmd) => {
                    kv.remove(cmd.key);
                    ("", 0, ResponseCode::Success)
                }
            },
            Err(_) => {
                let msg = "Cannot parse command!";
                (msg, msg.len(), ResponseCode::Error)
            }
        };

    // code is encode as 4 bytes in response
    const CODE_LEN: usize = 4;
    let total_len = tw_len + CODE_LEN;
    connection.write_buf[..HEADER_LEN].copy_from_slice(&(total_len as u32).to_le_bytes());
    connection.write_buf[HEADER_LEN..HEADER_LEN + CODE_LEN].copy_from_slice(&(code as u32).to_le_bytes());
    connection.write_buf[HEADER_LEN + CODE_LEN..HEADER_LEN + CODE_LEN + tw_len].copy_from_slice(to_write.as_bytes());
    connection.write_buf_size = HEADER_LEN + total_len;

    // Remove request from buffer
    // TODO: memmove sucks
    let remain = connection.read_buf_size - HEADER_LEN - len;
    if remain > 0 {
        connection
            .read_buf
            .copy_within(HEADER_LEN + len..HEADER_LEN + len + remain, 0);
    }
    connection.read_buf_size = remain;
    // Change connection to sending
    connection.state = ConnectionState::Responding;
    // TODO: Although we are pipelining reads, we aren't pipelining writes when we do this
    state_res(connection);

    // Continue loop if request was processed nicely
    connection.state == ConnectionState::Reading
}

fn main() {
    // TODO: We should have a Server struct that stores the global hash table
    // so we don't have to fucking that global hash table around (apparently called "tramp-data")
    let mut kv: KV = HashMap::new();

    println!("hello i'm a server!");
    let listener = TcpListener::bind("127.0.0.1:1234").expect("Failed to bind");
    listener
        .set_nonblocking(true)
        .expect("Cannot set listener to nonblocking");

    let mut tasks = VecDeque::new();
    // Event loop
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => accept_new_conn(&mut tasks, stream),
            // Just continue for WouldBlock case
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => (),
            Err(e) => panic!("encountered IO error: {e}"),
        }

        // Process active connections
        // TODO: This is  wrong LOL because we could be polling from listener for new connections
        while let Some(task) = tasks.pop_front() {
            if let Some(task) = connection_io(task, &mut kv) {
                tasks.push_back(task)
            }
        }
    }
}

fn accept_new_conn(v: &mut VecDeque<Connection>, stream: TcpStream) {
    stream
        .set_nonblocking(true)
        .expect("Failed to set stream to nonblocking");
    let connection = new_connection(stream);
    v.push_back(connection);
}

fn connection_io(mut task: Connection, kv: &mut KV) -> Option<Connection> {
    match task.state {
        ConnectionState::Reading => {
            state_req(&mut task, kv);
            Some(task)
        }
        ConnectionState::Responding => {
            state_res(&mut task);
            Some(task)
        }
        ConnectionState::End => None,
    }
}

fn state_req(task: &mut Connection, kv: &mut KV) {
    while try_fill_buffer(task, kv) {}
}

// TODO: return Results and just end connections when errors occur
fn try_fill_buffer(task: &mut Connection, kv: &mut KV) -> bool {
    let cur_buf_size = task.read_buf_size;
    assert!(cur_buf_size < task.read_buf.len());
    let cap = task.read_buf.len() - cur_buf_size;

    let amt_read;
    loop {
        match task
            .inner
            .read(&mut task.read_buf[cur_buf_size..(cur_buf_size + cap)])
        {
            Ok(rv) => {
                amt_read = rv;
                break;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => (),
            Err(e) => panic!("encountered IO error: {e}"),
        }
    }

    // TODO: error just panics but we could end

    if amt_read == 0 {
        if task.read_buf_size > 0 {
            eprintln!("Unexpected EOF");
        } else {
            eprintln!("EOF");
        }
        task.state = ConnectionState::End;
        return false;
    }
    task.read_buf_size += amt_read;
    assert!(task.read_buf_size <= task.read_buf.len());

    // While loop to handle pipelining
    while try_one_request(task, kv) {}

    task.state == ConnectionState::Reading
}

fn state_res(task: &mut Connection) {
    while try_flush_buffer(task) {}
}

fn try_flush_buffer(task: &mut Connection) -> bool {
    let amt_write;
    loop {
        let remain = task.write_buf_size - task.write_buf_sent;
        match task
            .inner
            .write(&task.write_buf[task.write_buf_sent..task.write_buf_sent + remain])
        {
            Ok(rv) => {
                amt_write = rv;
                break;
            }
            // Just retry
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => (),
            Err(e) => panic!("encountered IO Error: {e}"),
        }
    }

    // TODO: error just panics but we could end
    task.write_buf_sent += amt_write;
    assert!(task.write_buf_sent <= task.write_buf_size);
    if task.write_buf_sent == task.write_buf_size {
        // Response fully sent, go back
        task.state = ConnectionState::Reading;
        task.write_buf_sent = 0;
        task.write_buf_size = 0;
        return false;
    }

    true
}
