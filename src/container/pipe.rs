use std::io::Error as IoError;
use std::os::unix::io::RawFd;
use nix::unistd::{pipe};
use nix::Error::{Sys, InvalidPath};
use nix::errno::Errno::EPIPE;

use libc::{c_int, c_void};
use libc::funcs::posix88::unistd::{close, write};
use libc::consts::os::posix88::{EINTR, EAGAIN};


pub struct CPipe {
    reader: RawFd,
    writer: RawFd,
}

impl CPipe {
    pub fn new() -> Result<CPipe, IoError> {
        match pipe() {
            Ok((reader, writer)) => Ok(CPipe {
                reader: reader, writer: writer
            }),
            Err(Sys(code)) => Err(IoError::from_raw_os_error(code as i32)),
            Err(InvalidPath) => unreachable!(),
        }
    }
    pub fn reader_fd(&self) -> c_int {
        return self.reader;
    }
    pub fn wakeup(&self) -> Result<(), IoError> {
        let mut rc;
        loop {
            unsafe {
                rc = write(self.writer,
                    ['x' as u8].as_ptr() as *const c_void, 1);
            }
            let err = IoError::last_os_error().raw_os_error();
            if rc < 0 && (err == Some(EINTR) || err == Some(EAGAIN)) {
                continue
            }
            break;
        }
        if rc == 0 {
            return Err(IoError::from_raw_os_error(EPIPE as i32));
        } else if rc == -1 {
            return Err(IoError::last_os_error());
        }
        return Ok(());
    }
}

impl Drop for CPipe {
    fn drop(&mut self) {
        unsafe {
            close(self.reader);
            close(self.writer);
        }
    }
}
