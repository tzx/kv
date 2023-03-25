use std::{
    error::Error,
    net::{TcpListener, TcpStream},
    os::fd::{AsRawFd, RawFd},
};

#[repr(transparent)]
pub struct PqFd(libc::pollfd);

impl PqFd {
    fn new_listener_fd(fd: RawFd) -> Self {
        PqFd(libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        })
    }

    fn new_connection_fd(fd: RawFd) -> Self {
        PqFd(libc::pollfd {
            fd,
            events: libc::POLLIN | libc::POLLERR,
            revents: 0,
        })
    }

    fn change_events(&mut self, reading: bool) {
        let mut new_events = self.0.events;
        new_events &= !(libc::POLLIN | libc::POLLOUT);
        new_events |= if reading { libc::POLLIN } else { libc::POLLOUT };
        self.0.events = new_events;
    }

    fn is_active(&self) -> bool {
        self.0.revents != 0
    }
}

#[derive(Debug)]
pub struct PollError(i32);

impl Error for PollError {}

impl std::fmt::Display for PollError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Poll Error: {}", self.0)
    }
}

// streams[i] corresponds to the FD fds[i]
// i = 0 is the listener stream
pub struct PollQueue {
    fds: Vec<PqFd>,
}

impl PollQueue {
    pub fn new(listener: &TcpListener) -> Self {
        let lis_fd = listener.as_raw_fd();
        let lpqfd = PqFd::new_listener_fd(lis_fd);
        PollQueue { fds: vec![lpqfd] }
    }

    pub fn insert(&mut self, stream: &TcpStream) {
        let raw_fd = stream.as_raw_fd();
        let pqfd = PqFd::new_connection_fd(raw_fd);
        self.fds.push(pqfd);
    }

    fn poll(&mut self) -> Result<(), PollError> {
        const TIMEOUT: libc::c_int = 1000;
        let res = unsafe {
            libc::poll(
                std::mem::transmute(self.fds.as_mut_ptr()),
                self.fds.len() as libc::nfds_t,
                TIMEOUT,
            )
        };
        if res > 0 {
            Ok(())
        } else {
            Err(PollError(res))
        }
    }

    pub fn get_active_connections(&mut self) -> Result<Vec<&PqFd>, PollError> {
        self.poll()?;
        let actives = self.fds.iter().filter(|&fd| fd.is_active()).collect();
        Ok(actives)
    }
}
