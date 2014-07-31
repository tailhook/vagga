use super::linux::{dead_processes, MaskSignals, SIGCHLD};
use libc::consts::os::posix88::{SIGTERM, SIGINT, SIGQUIT};
use libc::funcs::posix88::signal::kill;

use libc::{c_int, pid_t};
use collections::treemap::TreeMap;

pub enum Event {
    Signal(c_int),
    Exit(String, pid_t, i32),
}

pub struct Monitor {
    processes: TreeMap<pid_t, String>,
    signalmask: MaskSignals,
    failed: bool,
    exit_code: Option<int>,
    singleton: bool,
}

impl Monitor {
    pub fn new(singleton: bool) -> Monitor {
        Monitor {
            processes: TreeMap::new(),
            signalmask: MaskSignals::new(),
            failed: false,
            singleton: singleton,
            exit_code: None,
        }
    }
    pub fn wait_all(&mut self) {
        while self.processes.len() > 0 {
            match self.next_event() {
                Exit(cname, pid, status) => {
                    if !self.singleton {
                        error!("Process {}:{} dead {}",
                            cname, pid, human_status(status));
                    }
                }
                Signal(sig)
                if sig == SIGTERM || sig == SIGINT || sig == SIGQUIT => {
                    debug!("Got {}. Propagating.", sig);
                    self.send_all(sig);
                }
                Signal(sig) => {
                    debug!("Got {}. Ignoring.", sig);
                }
            }
        }
    }
    pub fn send_all(&mut self, sig: i32) {
        if self.exit_code.is_none() && self.failed {
            self.exit_code = Some(128 + sig as int);
        }
        for (pid, name) in self.processes.iter() {
            unsafe {
                debug!("Sending {} to {}:{}", sig, name, pid);
                kill(*pid, sig);
            }
        }
    }
    pub fn fail(&mut self) {
        self.failed = true;
    }
    pub fn set_exit_status(&mut self, val: int) {
        self.exit_code = Some(val);
    }
    pub fn add(&mut self, name: String, pid: pid_t) {
        assert!(self.processes.insert(pid, name));
    }
    pub fn get_status(&self) -> int {
        return self.exit_code.unwrap_or(0);
    }
    pub fn ok(&self) -> bool {
        return !self.failed && self.processes.len() > 0;
    }
    pub fn next_event(&mut self) -> Event {
        loop {
            for (pid, status) in dead_processes() {
                let (pid, name) = match self.processes.find(&pid) {
                    Some(ref name) => (pid, (*name).clone()),
                    None => {
                        debug!("Unknown process {} exited with {}",
                            pid, status);
                        continue;
                    }
                };
                if self.processes.remove(&pid) && self.singleton {
                    // Translate OS exit code to bash-style exit code
                    if status & 0xff == 0 {
                        // Exit with status
                        self.exit_code = Some(((status >> 8) & 0xff) as int);
                    } else {
                        // Dead on signal
                        self.exit_code = Some((status & 0x7f) as int);
                    }
                }
                return Exit(name, pid, status);
            }
            let sig = self.signalmask.wait();
            if sig == SIGCHLD {
                continue;
            }
            return Signal(sig);
        }
    }
}

pub fn human_status(value: i32) -> String {
    // Note usually pid1 does a status translation too, so 128 + xxx are
    // signals too
    let exit = (value >> 8) & 0xff;
    if value & 0xff == 0 || exit & 0x80 == 0x80 {
        return format!("with exit status {}", (value >> 8) & 0xff);
    } else {
        return format!("on signal {}", value & 0x7f);
    }
}

