use std::io::process::Process;

pub use libc::consts::os::posix88::{SIGTERM, SIGINT, SIGQUIT, EINTR, ECHILD};
use libc::{c_int, pid_t};


static WNOHANG: c_int = 1;

#[deriving(Show)]
pub enum Signal {
    Terminate(int),  // Actual signal for termination: INT, TERM, QUIT...
    Child(pid_t, int),  //  pid and result code
}

#[deriving(Default)]
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

fn _convert_status(status: i32) -> int {
    if status & 0xff == 0 {
        return ((status & 0xff00) >> 8) as int;
    }
    return (128 + (status & 0x7f)) as int;  // signal
}

pub fn check_children() -> Option<Signal> {
    let mut status: i32 = 0;
    let pid = unsafe { waitpid(-1, &mut status, WNOHANG) };
    if pid > 0 {
        return Some(Child(pid, _convert_status(status)));
    }
    return None
}


pub fn send_signal(pid: pid_t, sig: int) {
    Process::kill(pid, sig).ok();
}
