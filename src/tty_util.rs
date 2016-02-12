use std::io;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::fs::File;

use libc::{c_int, pid_t};
use libc::{getpgrp};
use nix::sys::ioctl::ioctl;
use nix::unistd::{isatty, dup};


mod ffi {
    use libc::{c_ulong};

    pub static TIOCSPGRP: c_ulong = 21520;
}


#[derive(Debug)]
pub struct TtyGuard {
    tty: Option<File>,
    my_pgrp: pid_t,
    ownership: bool,
}

impl TtyGuard {
    pub fn take(&mut self) -> Result<(), io::Error> {
        let &mut TtyGuard { ref tty, ref mut ownership, my_pgrp } = self;
        tty.as_ref().map_or(Ok(()), |f| {
            try!(unsafe { give_tty_to(f.as_raw_fd(), my_pgrp) });
            *ownership = true;
            Ok(())
        })
    }
    pub fn give(&mut self, pid: pid_t) -> Result<(), io::Error> {
        let &mut TtyGuard { ref tty, ref mut ownership, .. } = self;
        tty.as_ref().map_or(Ok(()), |f| {
            try!(unsafe { give_tty_to(f.as_raw_fd(), pid) });
            *ownership = false;
            Ok(())
        })
    }
    pub fn capture_tty() -> Result<TtyGuard, io::Error> {
        for i in 0..2 {
            if isatty(i).unwrap_or(false) {
                // after we determined which FD is a TTY there is no way
                // to ensure that the same fd will be at the same number
                // So we duplicate it:
                let mut guard = TtyGuard {
                    tty: Some(unsafe { File::from_raw_fd(try!(dup(i))) }),
                    my_pgrp: unsafe { getpgrp() },
                    ownership: false,
                };
                try!(guard.take());
                return Ok(guard)
            }
        }
        Ok(TtyGuard {
            tty: None,
            my_pgrp: unsafe { getpgrp() },
            ownership: false,
        })
    }
}

impl Drop for TtyGuard {
    fn drop(&mut self) {
        if self.ownership {
            self.take().ok();
        }
    }
}

// dealing with file descriptors is always unsafe
unsafe fn give_tty_to(fd: c_int, pgrp: pid_t) -> Result<(), io::Error> {
    let res = ioctl(fd, ffi::TIOCSPGRP, &pgrp);
    if res == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}
