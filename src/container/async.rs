use std::io::Error as IoError;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;

use libc::{c_int};
use libc::consts::os::posix88::{EINTR, ETIMEDOUT, EAGAIN};
use libc::consts::os::posix88::{SIGTERM, SIGINT, SIGQUIT};
use time::Duration;

use super::signal;
use super::util::{Time, get_time};
use self::Event::*;

const SIGCHLD: c_int = 17;

#[derive(Debug)]
pub enum Event<Name> {
    Signal(signal::Signal),
    Timeout(Name),
    Input(Name),
}

struct FileDesc(c_int);

struct Unordered<T>(T);

impl<T> PartialOrd for Unordered<T> {
    fn partial_cmp(&self, _: &Unordered<T>) -> Option<Ordering> {
        Some(Ordering::Equal)
    }
}
impl<T> Ord for Unordered<T> {
    fn cmp(&self, _: &Unordered<T>) -> Ordering {
        Ordering::Equal
    }
}
impl<T> PartialEq for Unordered<T> {
    fn eq(&self, _: &Unordered<T>) -> bool { false }
}
impl<T> Eq for Unordered<T> {}

pub struct Loop<Name> {
    queue: BinaryHeap<(i64, Unordered<Name>)>,
    epoll_fd: FileDesc,
    signal_fd: FileDesc,
    inputs: HashMap<c_int, Name>,
}

extern {
    fn create_epoll() -> c_int;
    fn create_signalfd() -> c_int;
    fn close(fd: c_int) -> c_int;
    fn epoll_add_read(efd: c_int, fd: c_int) -> c_int;
    fn epoll_wait_read(epfd: c_int, timeout: c_int) -> c_int;
    fn read_signal(rd: c_int) -> c_int;
}

impl<Name: Clone> Loop<Name> {
    pub fn new() -> Result<Loop<Name>, IoError> {
        let efd = unsafe { create_epoll() };
        if efd < 0 {
            return Err(IoError::last_error());
        }
        let epoll = FileDesc(efd);
        let sfd = unsafe { create_signalfd() };
        if sfd < 0 {
            return Err(IoError::last_error());
        }
        let sig = FileDesc(sfd);
        if unsafe { epoll_add_read(efd, sfd) } < 0 {
            return Err(IoError::last_error());
        }
        return Ok(Loop {
            queue: BinaryHeap::new(),
            epoll_fd: epoll,
            signal_fd: sig,
            inputs: HashMap::new(),
        });
    }
    pub fn add_timeout(&mut self, duration: Duration, name: Name) {
        self.queue.push((-(get_time()*1000.) as i64, Unordered(name)));
    }
    fn get_timeout(&mut self) -> c_int {
        self.queue.peek()
            .map(|&(ts, _)| (-ts) - (get_time()*1000.0) as i64)
            .map(|ts| if ts >= 0 { ts } else { 0 })
            .unwrap_or(-1)
            as i32
    }
    pub fn poll(&mut self) -> Event<Name> {
        loop {
            if let Some(sig) = signal::check_children() {
                return Signal(sig);
            }
            let timeo = self.get_timeout();
            let FileDesc(sfd) = self.signal_fd;
            let FileDesc(efd) = self.epoll_fd;
            let fd = unsafe { epoll_wait_read(efd, timeo) };
            if fd == -ETIMEDOUT { // Timeout
                debug!("Timeout");
                if self.get_timeout() != 0 {
                    continue;
                }
                let (_, Unordered(name)) = self.queue.pop().unwrap();
                return Timeout(name);
            } else if fd == -EINTR {
                continue
            } else if fd < 0 {
                panic!(format!("Error in epoll: {}", IoError::last_error()));
            } else if fd == sfd { // Signal
                debug!("Signal");
                let rc =  unsafe { read_signal(sfd) };
                if rc == -EINTR || rc == -EAGAIN {
                    continue;
                } else if rc <= 0 {
                    panic!(format!("Error in read_signal: {}",
                        IoError::last_error()));
                } else {
                    match rc {
                        sig@SIGTERM | sig@SIGINT | sig@SIGQUIT => {
                            return Signal(
                                signal::Signal::Terminate(sig));
                        }
                        SIGCHLD => {
                            continue;  // Will waitpid on next iteration
                        }
                        _ => {
                            warn!("Signal {} ignored", rc);
                            continue;
                        }
                    }
                }
            } else {
                debug!("Input {}", fd);
                return Input(self.inputs[fd].clone());
            }
            unreachable!();
        }
    }
}

impl Drop for FileDesc {
    fn drop(&mut self) {
        let FileDesc(fd) = *self;
        unsafe { close(fd) };
    }
}
