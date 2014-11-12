use std::io::{IoError, EndOfFile};
use std::os::{Pipe, pipe};
use std::os::errno;

use libc::{c_int, c_void};
use libc::funcs::posix88::unistd::{close, write};
use libc::consts::os::posix88::{EINTR, EAGAIN};


pub struct CPipe(Pipe);

impl CPipe {
    pub fn new() -> Result<CPipe, IoError> {
        match unsafe { pipe() } {
            Ok(pipe) => Ok(CPipe(pipe)),
            Err(e) => Err(e),
        }
    }
    pub fn reader_fd(&self) -> c_int {
        let &CPipe(ref pipe) = self;
        return pipe.reader;
    }
    pub fn wakeup(&self) -> Result<(), IoError> {
        let mut rc;
        let &CPipe(ref pipe) = self;
        loop {
            unsafe {
                rc = write(pipe.writer, ['x' as u8].as_ptr() as *const c_void, 1);
            }
            if rc < 0 && (errno() as i32 == EINTR || errno() as i32 == EAGAIN) {
                continue
            }
            break;
        }
        if rc == 0 {
            return Err(IoError { kind: EndOfFile, detail: None,
                desc: "Pipe was closed. Probably process is dead"});
        } else if rc == -1 {
            return Err(IoError::last_error());
        }
        return Ok(());
    }
}

impl Drop for CPipe {
    fn drop(&mut self) {
        match self {
            &CPipe(ref pipe) => {
                unsafe {
                    close(pipe.reader);
                    close(pipe.writer);
                }
            }
        }
    }
}
