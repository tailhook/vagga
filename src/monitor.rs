use super::linux::{dead_processes, MaskSignals, SIGCHLD};
use libc::consts::os::posix88::{SIGTERM, SIGINT};
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
}

impl Monitor {
    pub fn new() -> Monitor {
        Monitor {
            processes: TreeMap::new(),
            signalmask: MaskSignals::new(),
            failed: false,
        }
    }
    pub fn wait_all(&mut self) {
        while self.processes.len() > 0 {
            match self.next_event() {
                Exit(_, _, _) => {
                    self.fail();
                }
                Signal(SIGTERM) => {
                    debug!("Got SIGTERM. Propagating.");
                    self.send_all(SIGTERM);
                }
                Signal(SIGINT) => {
                    debug!("Got SIGINT. Propagating.");
                    self.send_all(SIGINT);
                }
                Signal(sig) => {
                    debug!("Got {}. Ignoring.", sig);
                }
            }
        }
    }
    pub fn send_all(&self, sig: i32) {
        for (pid, _) in self.processes.iter() {
            unsafe {
                kill(*pid, sig);
            }
        }
    }
    pub fn fail(&mut self) {
        self.failed = true;
    }
    pub fn add(&mut self, name: String, pid: pid_t) {
        assert!(self.processes.insert(pid, name));
    }
    pub fn get_status(&self) -> int {
        return 0;
    }
    pub fn ok(&self) -> bool {
        return self.failed || self.processes.len() == 0;
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
                self.processes.remove(&pid);
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

