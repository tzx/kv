use std::net::{TcpListener, TcpStream};

use pollq::{PollQueue, PollError, PqFd};

use crate::connection::Connection;

// tasks[i] corresponds to connection in pollq[i + 1] since pollq[0] has the listener
pub struct TaskPollQueue {
    tasks: Vec<Connection>,
    pollq: PollQueue,
}

impl TaskPollQueue {
    pub fn new(listener: &TcpListener) -> Self {
        TaskPollQueue {
            tasks: vec![],
            pollq: PollQueue::new(listener),
        }
    }

    pub fn poll_actives(&mut self) -> Result<Vec<&PqFd>, PollError>{
        self.pollq.get_active_connections()
    }

    pub fn add_new_conn(&mut self, stream: TcpStream) {
        self.pollq.insert(&stream);

        let task = Connection::new(stream);
        self.tasks.push(task);
    }
}
