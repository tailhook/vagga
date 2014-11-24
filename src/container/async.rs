use std::io::IoError;
use std::collections::PriorityQueue;
use std::collections::HashMap;
use libc::c_int;

use super::signal;

pub enum Event<Name> {
    Signal(signal::Signal),
    Timeout(Name),
    Input(Name),
}

struct FileDesc(c_int);

struct Unordered<T>(T);

impl<T> PartialOrd for Unordered<T> {
    fn partial_cmp(&self, other: &Unordered<T>) -> Option<Ordering> { Some(Equal) }
}
impl<T> Ord for Unordered<T> {
    fn cmp(&self, other: &Unordered<T>) -> Ordering { Equal }
}
impl<T> PartialEq for Unordered<T> {
    fn eq(&self, other: &Unordered<T>) -> bool { false }
}
impl<T> Eq for Unordered<T> {}

struct Loop<Name> {
    queue: PriorityQueue<(i64, Unordered<Name>)>,
    epoll_fd: FileDesc,
    signal_fd: FileDesc,
    inputs: HashMap<c_int, Name>,
}

extern {
    fn create_epoll() -> c_int;
    fn create_signalfd() -> c_int;
    fn close(fd: c_int) -> c_int;
}

impl<Name> Loop<Name> {
    fn new() -> Result<Loop<Name>, IoError> {
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
        return Ok(Loop {
            queue: PriorityQueue::new(),
            epoll_fd: epoll,
            signal_fd: sig,
            inputs: HashMap::new(),
        });
    }
    fn poll(&mut self) -> Event<Name> {
        unimplemented!();
    }
}

impl Drop for FileDesc {
    fn drop(&mut self) {
        let FileDesc(fd) = *self;
        unsafe { close(fd) };
    }
}
