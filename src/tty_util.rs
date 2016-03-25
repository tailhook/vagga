use std::io;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::fs::File;

use libc::{c_int, pid_t};
use libc::{getpgrp};
use nix::sys::ioctl::ioctl;
use nix::unistd::{isatty, dup};


mod ffi {
    use libc::{c_ulong};

    pub static TIOCSPGRP: c_ulong = 0x5410;
    pub static TIOCGPGRP: c_ulong = 0x540F;
}


#[derive(Debug)]
pub struct TtyGuard {
    tty: Option<File>,
    my_pgrp: pid_t,
}

impl TtyGuard {
    pub fn take(&mut self) -> Result<(), io::Error> {
        let &mut TtyGuard { ref tty, my_pgrp, .. } = self;
        tty.as_ref().map_or(Ok(()), |f| {
            try!(unsafe { give_tty_to(f.as_raw_fd(), my_pgrp) });
            Ok(())
        })
    }
    pub fn give(&mut self, pid: pid_t) -> Result<(), io::Error> {
        let &mut TtyGuard { ref tty, .. } = self;
        tty.as_ref().map_or(Ok(()), |f| {
            try!(unsafe { give_tty_to(f.as_raw_fd(), pid) });
            Ok(())
        })
    }
    /// Check terminal and take it if it's not owned
    pub fn check(&mut self) -> Result<(), io::Error> {
        let &mut TtyGuard { ref tty, my_pgrp, .. } = self;
        tty.as_ref().map_or(Ok(()), |f| {
            if try!(unsafe { get_group(f.as_raw_fd()) }) == 0 {
                try!(unsafe { give_tty_to(f.as_raw_fd(), my_pgrp) });
            }
            Ok(())
        })
    }
    pub fn capture_tty() -> Result<TtyGuard, io::Error> {
        for i in 0..3 {
            if isatty(i).unwrap_or(false) {
                // after we determined which FD is a TTY there is no way
                // to ensure that the same fd will be at the same number
                // So we duplicate it:
                let mut guard = TtyGuard {
                    tty: Some(unsafe { File::from_raw_fd(try!(dup(i))) }),
                    my_pgrp: unsafe { getpgrp() },
                };
                try!(guard.take());
                return Ok(guard)
            }
        }
        Ok(TtyGuard {
            tty: None,
            my_pgrp: unsafe { getpgrp() },
        })
    }
}

impl Drop for TtyGuard {
    fn drop(&mut self) {
        self.take().ok();
    }
}

// dealing with file descriptors is always unsafe
unsafe fn get_group(fd: c_int)
    -> Result<pid_t, io::Error>
{
    let mut pgrp = 0;
    let res = ioctl(fd, ffi::TIOCGPGRP, &mut pgrp);
    if res == 0 {
        Ok(pgrp)
    } else {
        Err(io::Error::last_os_error())
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
