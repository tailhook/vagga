use std::ptr::null;
use std::io::{IoError, EndOfFile};
use std::os::{Pipe, pipe};
use std::os::errno;

use libc::{c_int, c_void};
use libc::funcs::posix88::unistd::{close, write};
use libc::consts::os::posix88::{EINTR, EAGAIN};

extern {
    fn openpty(master: *mut c_int, slave: *mut c_int, name: *const c_void,
        termp: *const c_void, winp: *const c_void) -> c_int;
    fn set_cloexec(fd: c_int, flag: c_int);
}

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
            &mut CPipe(ref pipe) => {
                unsafe {
                    close(pipe.reader);
                    close(pipe.writer);
                }
            }
        }
    }
}


pub struct Pty {
    master_fd: c_int,
    slave_fd: c_int,
}

impl Pty {
    pub fn new() -> Result<Pty, IoError> {
        let mut master: c_int = 0;
        let mut slave: c_int = 0;
        if unsafe { openpty(&mut master, &mut slave,
            null(), null(), null()) } < 0
        {
            return Err(IoError::last_error());
        }
        unsafe { set_cloexec(master, 1) };
        unsafe { set_cloexec(slave, 1) };
        return Ok(Pty {
            master_fd: master,
            slave_fd: slave,
        });
    }
}

impl Drop for Pty {
    fn drop(&mut self) {
        unsafe { close(self.master_fd) };
        unsafe { close(self.slave_fd) };
    }
}
