use std::rc::Rc;
use std::fmt::{Show, Formatter, FormatError};
use std::cell::RefCell;
use std::collections::TreeMap;
use std::collections::HashMap;
use std::collections::PriorityQueue;
use std::mem::swap;
use std::time::Duration;
use libc::pid_t;
use time::{Timespec, get_time};

use super::container::Command;
use super::signal;
use super::async::{Loop, Event, Signal, Timeout, Input};

type ProcRef<'a> = Rc<RefCell<Process<'a>>>;

pub enum MonitorResult {
    Killed,
    Reboot,
}

pub enum PrepareResult {
    Run,
    Error(String),
    Shutdown,
}

pub trait Executor {
    fn prepare(&self) -> PrepareResult { return Run; }
    fn command(&self) -> Command;
    fn finish(&self, _status: int) -> bool { return true; }
}

pub struct Process<'a> {
    name: Rc<String>,
    current_pid: Option<pid_t>,
    start_time: Option<Timespec>,
    executor: Box<Executor + 'a>,
}

impl<'a> Show for Process<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FormatError> {
        "Signal(".fmt(fmt)
        .and(self.name.fmt(fmt))
        .and(")".fmt(fmt))
    }
}

pub struct Monitor<'a> {
    processes: Vec<ProcRef<'a>>,
    pids: HashMap<pid_t, ProcRef<'a>>,
    aio: Loop<ProcRef<'a>>,
}

impl<'a> Show for Event<Rc<RefCell<Process<'a>>>> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FormatError> {
        match self {
            &Signal(ref sig) => {
                fmt.write("Signal(".as_bytes())
                .and(sig.fmt(fmt))
                .and(fmt.write(")".as_bytes()))
            }
            &Timeout(ref name) => {
                fmt.write("Timeout(".as_bytes())
                .and(name.borrow().fmt(fmt))
                .and(fmt.write(")".as_bytes()))
            }
            &Input(ref name) => {
                fmt.write("Input(".as_bytes())
                .and(name.borrow().fmt(fmt))
                .and(fmt.write(")".as_bytes()))
            }
        }
    }
}

impl<'a> Monitor<'a> {
    pub fn new<'x>() -> Monitor<'x> {
        return Monitor {
            processes: Vec::new(),
            pids: HashMap::new(),
            aio: Loop::new().unwrap(),
        };
    }
    pub fn add(&mut self, name: Rc<String>, executor: Box<Executor>)
    {
        let prc = Rc::new(RefCell::new(Process {
            name: name,
            current_pid: None,
            start_time: None,
            executor: executor}));
        self.processes.push(prc.clone());
        self.aio.add_timeout(Duration::seconds(0), prc);
    }
    fn _start_process(&mut self, prc: ProcRef) -> PrepareResult {
        let prepare_result = prc.borrow().executor.prepare();
        match prepare_result {
            Run => {
                match prc.borrow().executor.command().spawn() {
                    Ok(pid) => {
                        info!("Process {} started with pid {}",
                            prc.borrow().name, pid);
                        self.pids.insert(pid, prc.clone());
                    }
                    Err(e) => {
                        error!("Can't run container {}: {}",
                            prc.borrow().name, e);
                        prc.borrow().executor.finish(-1);
                        return Shutdown;
                    }
                }
            }
            Error(_) => {
                return Shutdown;
            }
            _ => {}
        }
        return prepare_result;
    }
    fn _reap_child(&mut self, prc: ProcRef, pid: pid_t, status: int)
        -> bool
    {
        warn!("Child {}:{} exited with status {}",
            prc.borrow().name, pid, status);
        prc.borrow().executor.finish(status);
        return false;
    }
    pub fn run(&mut self) -> MonitorResult {
        debug!("Starting with {} processes",
            self.processes.len());
        // Main loop
        loop {
            let sig = self.aio.poll();
            info!("Got signal {}", sig);
            match sig {
                Input(_) => {
                    unimplemented!();
                }
                Timeout(prc) => {
                    self._start_process(prc);
                }
                Signal(signal::Terminate(sig)) => {
                    for prc in self.processes.iter() {
                        match prc.borrow().current_pid {
                            Some(pid) => signal::send_signal(pid, sig),
                            None => {}
                        }
                    }
                    break;
                }
                Signal(signal::Child(pid, status)) => {
                    let prc = match self.pids.pop(&pid) {
                        Some(name) => name,
                        None => {
                            warn!("Unknown process {} dead with {}",
                                pid, status);
                            continue;
                        },
                    };
                    if !self._reap_child(prc, pid, status) {
                        break;
                    }
                }
            }
        }
        // TODO(tailhook) self.start_queue.clear();
        // Shut down loop
        let mut processes = Vec::new();
        swap(&mut processes, &mut self.processes);
        let mut left: TreeMap<pid_t, ProcRef> = processes.into_iter()
            .filter(|prc| prc.borrow().current_pid.is_some())
            .map(|prc| (prc.borrow().current_pid.unwrap(), prc))
            .collect();
        info!("Shutting down, {} processes left",
              left.len());
        while left.len() > 0 {
            let sig = self.aio.poll();
            info!("Got signal {}", sig);
            match sig {
                Input(_) => {
                    unimplemented!();
                }
                Timeout(prc) => {
                    unimplemented!();
                }
                Signal(signal::Terminate(sig)) => {
                    for (_name, prc) in left.iter() {
                        match prc.borrow().current_pid {
                            Some(pid) => signal::send_signal(pid, sig),
                            None => {}
                        }
                    }
                }
                Signal(signal::Child(pid, status)) => {
                    match left.pop(&pid) {
                        Some(prc) => {
                            info!("Child {}:{} exited with status {}",
                                prc.borrow().name, pid, status);
                        }
                        None => {
                            warn!("Unknown process {} dead with {}",
                                pid, status);
                        }
                    }
                }
            }
        }
        return Killed;
    }
}
