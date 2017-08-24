use std::io;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::fs::File;

use libc::{c_int, pid_t};
use libc::{getpgrp, kill, ioctl};
use nix;
use nix::errno::Errno;
use nix::unistd::{isatty, dup};


#[cfg(target_env="musl")]
mod ffi {
    use libc::{c_int};

    pub static TIOCSPGRP: c_int = 0x5410;
    pub static TIOCGPGRP: c_int = 0x540F;
}

#[cfg(target_env="gnu")]
mod ffi {
    pub static TIOCSPGRP: u64 = 0x5410;
    pub static TIOCGPGRP: u64 = 0x540F;
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
            unsafe { give_tty_to(f.as_raw_fd(), my_pgrp) }?;
            Ok(())
        })
    }
    pub fn give(&mut self, pid: pid_t) -> Result<(), io::Error> {
        let &mut TtyGuard { ref tty, .. } = self;
        tty.as_ref().map_or(Ok(()), |f| {
            unsafe { give_tty_to(f.as_raw_fd(), pid) }?;
            Ok(())
        })
    }
    /// Check terminal and take it if it's not owned
    pub fn check(&mut self) -> Result<(), io::Error> {
        let &mut TtyGuard { ref tty, my_pgrp, .. } = self;
        tty.as_ref().map_or(Ok(()), |f| {
            let tty_owner_grp = unsafe { get_group(f.as_raw_fd()) }?;
            if tty_owner_grp != 0 {
                let kill_res = unsafe { kill(tty_owner_grp, 0) };
                if kill_res < 0 && Errno::last() == Errno::ESRCH {
                    unsafe { give_tty_to(f.as_raw_fd(), my_pgrp) }?;
                }
            }
            Ok(())
        })
    }
    pub fn new() -> Result<TtyGuard, nix::Error> {
        let my_pgrp = unsafe { getpgrp() };
        if my_pgrp != 0 {
            // my_pgrp can be zero if group owner is outside of the PID ns
            for i in 0..3 {
                if isatty(i).unwrap_or(false) {
                    // after we determined which FD is a TTY there is no way
                    // to ensure that the same fd will be at the same number
                    // So we duplicate it:
                    let guard = TtyGuard {
                        tty: Some(unsafe { File::from_raw_fd(dup(i)?) }),
                        my_pgrp: unsafe { getpgrp() },
                    };
                    return Ok(guard)
                }
            }
        }
        Ok(TtyGuard {
            tty: None,
            my_pgrp: my_pgrp,
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
