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
    Exit(int),
}

pub enum MonitorStatus {
    Run,
    Error(String),
    Shutdown(int),
}

pub trait Executor {
    fn prepare(&self) -> MonitorStatus { return Run; }
    fn command(&self) -> Command;
    fn finish(&self, status: int) -> MonitorStatus { return Shutdown(status); }
}

pub struct Process<'a> {
    name: Rc<String>,
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
    status: Option<int>,
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
            status: None,
        };
    }
    pub fn add(&mut self, name: Rc<String>, executor: Box<Executor>)
    {
        let prc = Rc::new(RefCell::new(Process {
            name: name,
            start_time: None,
            executor: executor}));
        self.processes.push(prc.clone());
        self.aio.add_timeout(Duration::seconds(0), prc);
    }
    fn _start_process(&mut self, prc: ProcRef) -> MonitorStatus {
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
                        return prc.borrow().executor.finish(127);
                    }
                }
            }
            Error(_) => {
                return Shutdown(127);
            }
            _ => {}
        }
        return prepare_result;
    }
    fn _reap_child(&mut self, prc: ProcRef, pid: pid_t, status: int)
        -> MonitorStatus
    {
        warn!("Child {}:{} exited with status {}",
            prc.borrow().name, pid, status);
        return prc.borrow().executor.finish(status);
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
                    match self._start_process(prc) {
                        Shutdown(x) => {
                            self.status = Some(x);
                            break;
                        }
                        Error(_) => unreachable!(),
                        Run => {}
                    }
                }
                Signal(signal::Terminate(sig)) => {
                    for (pid, _) in self.pids.iter() {
                        signal::send_signal(*pid, sig);
                    }
                    break;
                }
                Signal(signal::Child(pid, status)) => {
                    let prc = match self.pids.pop(&pid) {
                        Some(prc) => prc,
                        None => {
                            warn!("Unknown process {} dead with {}",
                                pid, status);
                            continue;
                        },
                    };
                    match self._reap_child(prc, pid, status) {
                        Shutdown(x) => {
                            self.status = Some(x);
                            break;
                        }
                        Error(_) => unreachable!(),
                        Run => {}
                    }
                }
            }
        }
        // TODO(tailhook) self.start_queue.clear();
        // Shut down loop
        info!("Shutting down, {} processes left",
              self.pids.len());
        while self.pids.len() > 0 {
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
                    for (pid, _) in self.pids.iter() {
                        signal::send_signal(*pid, sig);
                    }
                }
                Signal(signal::Child(pid, status)) => {
                    let prc = match self.pids.pop(&pid) {
                        Some(prc) => prc,
                        None => {
                            warn!("Unknown process {} dead with {}",
                                pid, status);
                            continue;
                        }
                    };
                    info!("Child {}:{} exited with status {}",
                        prc.borrow().name, pid, status);
                    match self._reap_child(prc, pid, status) {
                        Shutdown(x) => {
                            self.status = Some(x);
                        }
                        Error(_) => unreachable!(),
                        Run => {}
                    }
                }
            }
        }
        match self.status {
            Some(val) => Exit(val),
            None => Killed,
        }
    }
}
