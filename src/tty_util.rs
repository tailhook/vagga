use libc::{c_int, pid_t};
use libc::{signal};
use libc::STDIN_FILENO;
use nix;
use nix::errno::EINTR;
use nix::sys::ioctl::ioctl;
use nix::sys::signal::{kill, SIGCONT, SIGTTIN, SIGTTOU};
use nix::sys::wait::{waitpid, WaitStatus, WNOHANG, WUNTRACED};
use nix::unistd::{isatty, getpid, setpgid};

use unshare::{Child, Command, ExitStatus};


mod ffi {
    use libc::{c_ulong, pid_t, sighandler_t};

    pub static WAIT_ANY: pid_t = -1;

    pub static SIG_IGN: sighandler_t = 1;
    pub static SIG_ERR: sighandler_t = !0;

    pub static TIOCSPGRP: c_ulong = 21520;
}


pub struct TtyGuard {
    tty_fd: c_int,
    pgrp: pid_t,
}

impl TtyGuard {
    pub fn new(tty_fd: c_int, pgrp: pid_t) -> Result<TtyGuard, String> {
        try!(give_tty_to(tty_fd, pgrp));
        Ok(TtyGuard {
            tty_fd: tty_fd,
            pgrp: pgrp,
        })
    }

    pub fn wait_child(&self, cmd: &Command, child: &Child)
        -> Result<Option<ExitStatus>, String>
    {
        loop {
            match waitpid(ffi::WAIT_ANY, Some(WNOHANG | WUNTRACED)) {
                Ok(WaitStatus::Stopped(_, signum)) => {
                    if signum == SIGTTOU || signum == SIGTTIN {
                        try!(give_tty_to(self.tty_fd, child.pid()));
                        kill(-child.pid(), SIGCONT).ok();
                    }
                    continue;
                },
                Ok(WaitStatus::Continued(_)) => {
                    continue;
                },
                Ok(WaitStatus::Exited(child_pid, st)) => {
                    if child_pid == child.pid() {
                        return Ok(Some(ExitStatus::Exited(st)));
                    }
                    continue;
                },
                Ok(WaitStatus::Signaled(child_pid, signum, core)) => {
                    if child_pid == child.pid() {
                        return Ok(Some(ExitStatus::Signaled(signum, core)));
                    }
                    continue;
                },
                Ok(WaitStatus::StillAlive) => {
                    return Ok(None);
                },
                Err(nix::Error::Sys(EINTR)) => {
                    continue;
                },
                Err(e) => {
                    return Err(
                        format!("Error when waiting for {:?}: {}", cmd, e));
                },
            }
        }
    }
}

impl Drop for TtyGuard {
    fn drop(&mut self) {
        let _ = give_tty_to(self.tty_fd, self.pgrp);
    }
}

pub fn prepare_tty() -> Result<Option<c_int>, String> {
    let tty_fd = STDIN_FILENO;
    let is_interactive = isatty(tty_fd).unwrap_or(false);
    if is_interactive {
        try!(ignore_tty_signals());
        let pid = getpid();
        try!(setpgid(pid, pid).map_err(|e| format!("{}", e)));
        try!(give_tty_to(tty_fd, pid));
        Ok(Some(tty_fd))
    } else {
        Ok(None)
    }
}

pub fn give_tty_to(fd: c_int, pgrp: pid_t) -> Result<(), String> {
    let res = unsafe { ioctl(fd, ffi::TIOCSPGRP, &pgrp) };
    match res {
        res if res < 0 => Err(
            format!("Error when giving tty with fd {} to process group {}",
                    fd, pgrp)),
        _ => Ok(()),
    }
}

pub fn ignore_tty_signals() -> Result<(), String> {
    try!(ignore_signal(SIGTTIN));
    try!(ignore_signal(SIGTTOU));
    Ok(())
}

fn ignore_signal(signum: i32) -> Result<(), String> {
    let res = unsafe { signal(signum, ffi::SIG_IGN) };
    if res == ffi::SIG_ERR {
        return Err(
            format!("Error when setting signal handler for signum: {}", signum));
    }
    Ok(())
}
