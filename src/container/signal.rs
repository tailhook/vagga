use std::io::Error as IoError;

use nix::sys::signal::kill;
use nix::errno::errno;
pub use libc::consts::os::posix88::{SIGTERM, SIGINT, SIGQUIT, EINTR, ECHILD};
use libc::{c_int, pid_t};
use self::Signal::*;


static WNOHANG: c_int = 1;

#[derive(Debug)]
pub enum Signal {
    Terminate(i32),  // Actual signal for termination: INT, TERM, QUIT...
    Child(pid_t, i32),  //  pid and result code
}

#[derive(Default)]
#[repr(C)]
struct CSignalInfo {
    signo: c_int,
    pid: pid_t,
    status: c_int,
}

extern {
    fn block_all_signals();
    fn waitpid(pid: pid_t, status: *mut c_int, options: c_int) -> pid_t;
}

pub fn block_all() {
    unsafe { block_all_signals() };
}

fn _convert_status(status: i32) -> i32 {
    if status & 0xff == 0 {
        return ((status & 0xff00) >> 8);
    }
    return (128 + (status & 0x7f));  // signal
}

pub fn check_children() -> Option<Signal> {
    let mut status: i32 = 0;
    let pid = unsafe { waitpid(-1, &mut status, WNOHANG) };

    if pid > 0 {
        return Some(Child(pid, _convert_status(status)));
    }
    return None
}

pub fn wait_process(pid: pid_t) -> Result<i32, IoError> {
    let mut status: i32 = 0;
    loop {
        let pid = unsafe { waitpid(pid, &mut status, 0) };
        if pid > 0 {
            return Ok(_convert_status(status));
        }
        if errno() == EINTR {
            continue;
        }
        return Err(IoError::last_error());
    }
}


pub fn send_signal(pid: pid_t, sig: i32) {
    kill(pid, sig as isize).ok();
}
