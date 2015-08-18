use std::rc::Rc;
use std::fmt::{Debug, Formatter};
use std::fmt::Error as FormatError;
use std::cell::RefCell;
use std::collections::HashMap;

use libc::pid_t;
use time::Duration;

use super::container::Command;
use super::signal;
use super::signal::Signal as Sig;
use super::util::{Time, get_time};
use super::async::{Loop};
use super::async::Event::{Signal, Timeout, Input};
use self::MonitorStatus::*;
use self::MonitorResult::*;

type ProcRef<'a> = Rc<RefCell<Process<'a>>>;

pub enum MonitorResult {
    Killed,
    Exit(i32),
}

pub enum MonitorStatus {
    Run,
    Error(String),
    Shutdown(i32),
}

pub trait Executor {
    fn prepare(&mut self) -> MonitorStatus { Run }
    fn command(&mut self) -> Command;
    fn finish(&mut self, status: i32) -> MonitorStatus { Shutdown(status) }
}

pub struct RunOnce {
    command: Option<Command>,
}

impl Executor for RunOnce {
    fn command(&mut self) -> Command {
        return self.command.take().expect("Command can't be run twice");
    }
}

/*
impl RunOnce {
    pub fn new(cmd: Command) -> RunOnce {
        RunOnce {
            command: Some(cmd),
        }
    }
}
*/


pub struct Process<'a> {
    name: Rc<String>,
    start_time: Option<Time>,
    executor: Box<Executor + 'a>,
}

impl<'a> Debug for Process<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FormatError> {
        write!(fmt, "Signal({:?})", self.name)
    }
}

pub struct Monitor<'a> {
    processes: Vec<ProcRef<'a>>,
    pids: HashMap<pid_t, ProcRef<'a>>,
    aio: Loop<ProcRef<'a>>,
    status: Option<i32>,
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
    pub fn add(&mut self, name: Rc<String>, executor: Box<Executor + 'a>)
    {
        let prc = Rc::new(RefCell::new(Process {
            name: name,
            start_time: None,
            executor: executor}));
        self.processes.push(prc.clone());
        self.aio.add_timeout(Duration::seconds(0), prc);
    }
    fn _start_process(&mut self, prc: ProcRef<'a>) -> MonitorStatus {
        let prepare_result = prc.borrow_mut().executor.prepare();
        match prepare_result {
            Run => {
                let mut pref = prc.borrow_mut();
                pref.start_time = Some(get_time());
                let spawn_result = pref.executor.command().spawn();
                match spawn_result {
                    Ok(pid) => {
                        info!("Process {} started with pid {}",
                            pref.name, pid);
                        self.pids.insert(pid, prc.clone());
                    }
                    Err(e) => {
                        error!("Can't run container {}: {}", pref.name, e);
                        return pref.executor.finish(127);
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
    fn _reap_child(&mut self, prc: ProcRef, pid: pid_t, status: i32)
        -> MonitorStatus
    {
        let mut prc = prc.borrow_mut();
        let start_time = prc.start_time.take().unwrap();
        warn!("Child {}:{} exited with status {} in {}s",
            prc.name, pid, status, get_time() - start_time);
        return prc.executor.finish(status);
    }
    fn _find_by_name(&self, name: &Rc<String>) -> Option<ProcRef<'a>> {
        for prc in self.processes.iter() {
            if prc.borrow().name == *name {
                return Some(prc.clone());
            }
        }
        return None;
    }
    pub fn force_start(&mut self, name: Rc<String>) -> Result<pid_t, String> {
        let prc = try!(self._find_by_name(&name)
            .ok_or("Process not found".to_string()));
        self._start_process(prc.clone());
        for (pid, pprc) in self.pids.iter() {
            if pprc.borrow().name == name {
                return Ok(*pid);
            }
        }
        return Err("Can't run command".to_string());
    }
    pub fn run(&mut self) -> MonitorResult {
        debug!("Starting with {} processes",
            self.processes.len());
        // Main loop
        loop {
            let sig = self.aio.poll();
            info!("Got signal {:?}", sig);
            match sig {
                Input(_) => {
                    unimplemented!();
                }
                Timeout(prc) => {
                    if prc.borrow().start_time.is_some() {
                        continue;  // Already started, e.g. by force_start
                    }
                    match self._start_process(prc) {
                        Shutdown(x) => {
                            self.status = Some(x);
                            for (pid, _) in self.pids.iter() {
                                signal::send_signal(*pid, signal::SIGTERM);
                            }
                            break;
                        }
                        Error(_) => unreachable!(),
                        Run => {}
                    }
                }
                Signal(Sig::Terminate(sig)) => {
                    for (pid, _) in self.pids.iter() {
                        signal::send_signal(*pid, sig);
                    }
                    break;
                }
                Signal(Sig::Child(pid, status)) => {
                    let prc = match self.pids.remove(&pid) {
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
                            for (pid, _) in self.pids.iter() {
                                signal::send_signal(*pid, signal::SIGTERM);
                            }
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
            info!("Got signal {:?}", sig);
            match sig {
                Input(_) => {
                    unimplemented!();
                }
                Timeout(_) => {
                    continue;
                }
                Signal(Sig::Terminate(sig)) => {
                    for (pid, _) in self.pids.iter() {
                        signal::send_signal(*pid, sig);
                    }
                }
                Signal(Sig::Child(pid, status)) => {
                    let prc = match self.pids.remove(&pid) {
                        Some(prc) => prc,
                        None => {
                            warn!("Unknown process {} dead with {}",
                                pid, status);
                            continue;
                        }
                    };
                    info!("Child {}:{} exited with status {:?}",
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

    pub fn run_command(cmd: Command) -> MonitorResult {
        let mut mon = Monitor::new();
        mon.add(Rc::new(cmd.name.to_string()),
                Box::new(RunOnce { command: Some(cmd) }));
        return mon.run();
    }
}
