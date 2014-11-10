use std::os::errno;
use std::io::IoError;
use std::ptr::null;
use std::num::zero;
use std::time::duration::Duration;
use std::default::Default;
use std::io::process::Process;
use libc::types::os::common::posix01::timespec;
use time::{Timespec, get_time};

pub use libc::consts::os::posix88::{SIGTERM, SIGINT, SIGQUIT, EINTR, ECHILD};
use libc::{c_int, pid_t};


static SIGCHLD: c_int = 17;
static WNOHANG: c_int = 1;

#[deriving(Show)]
pub enum Signal {
    Terminate(int),  // Actual signal for termination: INT, TERM, QUIT...
    Child(pid_t, int),  //  pid and result code
    Reboot,
    Timeout,  // Not actually a OS signal, but it's a signal for our app
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
    fn wait_any_signal(ptr: *mut CSignalInfo, timeout: *const timespec)
        -> c_int;
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

pub fn wait_next(reboot_supported: bool, timeout: Option<Timespec>) -> Signal {
    let mut status: i32 = 0;
    let pid = unsafe { waitpid(-1, &mut status, WNOHANG) };
    if pid > 0 {
        return Child(pid, _convert_status(status));
    }
    loop {
        let mut ptr = Default::default();
        let res = match timeout {
            Some(tm) => {
                let dur = tm - get_time();
                let c_dur = if dur < zero() {
                    timespec { tv_sec: 0, tv_nsec: 0 }
                } else {
                    timespec {
                        tv_sec: dur.num_seconds(),
                        tv_nsec: (dur - Duration::seconds(dur.num_seconds()))
                            .num_nanoseconds().unwrap(),
                    }
                };
                unsafe { wait_any_signal(&mut ptr, &c_dur) }
            }
            None => {
                unsafe { wait_any_signal(&mut ptr, null()) }
            }
        };
        if res != 0 {
            //  Any error is ok, because application should be always prepared
            //  for spurious timeouts
            //  only EAGAIN and EINTR expected
            return Timeout;
        }
        match ptr.signo {
            SIGCHLD => {
                loop {
                    status = 0;
                    let rc = unsafe { waitpid(ptr.pid, &mut status, WNOHANG) };
                    if rc < 0 {
                        if errno() == EINTR as int {
                            continue;
                        }
                        if errno() != ECHILD as int {
                            fail!("Failure '{}' not expected, on death of {}",
                                IoError::last_error(), ptr.pid);
                        }
                    } else {
                        assert_eq!(rc, ptr.pid);
                        assert_eq!(_convert_status(status), ptr.status as int);
                    }
                    break;
                }
                return Child(ptr.pid, ptr.status as int);
            }
            SIGQUIT if reboot_supported => {
                return Reboot;
            }
            sig@SIGTERM | sig@SIGINT | sig@SIGQUIT => {
                return Terminate(sig as int);
            }
            _ => continue,   // TODO(tailhook) improve logging
        }
    }
}

pub fn send_signal(pid: pid_t, sig: int) {
    Process::kill(pid, sig).ok();
}
