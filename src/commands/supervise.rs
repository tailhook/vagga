use std::io::stdio::{stdout, stderr};
use std::io::fs::readlink;

use libc::pid_t;
use libc::consts::os::posix88::{SIGTERM, SIGINT, SIGQUIT};
use collections::treemap::TreeMap;
use argparse::ArgumentParser;

use super::super::env::{Environ, Container};
use super::super::options::env_options;
use super::super::build::{build_container, link_container};
use super::super::config::{Shell, Plain, Supervise};
use super::super::config::{StopOnFailure, WaitAll, Restart};
use super::super::config::Command;
use super::super::commands::shell::exec_shell_command;
use super::super::commands::command::exec_plain_command;
use super::super::monitor::{Monitor, Signal, Exit};


pub fn run_supervise_command(env: &mut Environ, cmdname: &String,
    args: Vec<String>)
    -> Result<int, String>
{
    {
        let mut ap = ArgumentParser::new();
        env_options(env, &mut ap);
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => return Ok(122),
        }
    }
    let command = env.config.commands.find(cmdname).unwrap();
    let mut work_dir = env.work_dir.clone();
    if command.work_dir.is_some() {
        let ncwd = env.project_root.join(
            command.work_dir.as_ref().unwrap().as_slice());
        if !env.project_root.is_ancestor_of(&ncwd) {
            return Err(format!("Command's work-dir must be relative to \
                project root"));
        }
        work_dir = ncwd;
    }
    let (mode, processes) = match command.execute {
        Supervise(ref mode, ref processes) => (mode, processes),
        _ => unreachable!(),
    };
    let mut containers = TreeMap::new();
    for (cmdname, command) in processes.iter() {
        let cname = command.container.as_ref().unwrap();
        let mut container = try!(env.get_container(cname));
        if env.settings.version_check {
            try!(build_container(env, &mut container, false));
            try!(link_container(env, &container));
        } else {
            let lnk = env.local_vagga.join(container.fullname.as_slice());
            container.container_root = match readlink(&lnk) {
                Ok(path) => Some(lnk.dir_path().join(path)),
                Err(e) => return Err(format!("Container {} not found: {}",
                                             container.fullname, e)),
            };
        }
        if command.work_dir.is_some() {
            let ncwd = env.project_root.join(
                command.work_dir.as_ref().unwrap().as_slice());
            if !env.project_root.is_ancestor_of(&ncwd) {
                return Err(format!("The `work-dir` of {} of must be relative \
                    to project root", cmdname));
            }
        }
        containers.insert(cname.clone(), container);
    }

    let mut monitor = Monitor::new();

    let start = |cname: &String, monitor: &mut Monitor| -> bool {
        let command = processes.find(cname).unwrap();
        let fun = match command.execute {
            Shell(_) => exec_shell_command,
            Plain(_) => exec_plain_command,
            Supervise(_, _) => exec_supervise_command,
        };
        let container = containers.find(
            command.container.as_ref().unwrap()).unwrap();

        let cmdworkdir = if command.work_dir.is_some() {
            env.project_root.join(
                command.work_dir.as_ref().unwrap().as_slice())
        } else {
            work_dir.clone()
        };
        match fun(env, &cmdworkdir, command, container) {
            Ok(pid) => {
                info!("Command {} started with pid {}",
                    cname, pid);
                monitor.add(cname.clone(), pid);
                return true;
            }
            Err(e) => {
                error!("Error starting {}: {}", cname, e);
                monitor.send_all(SIGTERM);
                monitor.fail();
                return false;
            }
        }
    };

    for (cname, _) in processes.iter() {
        if !start(cname, &mut monitor) {
            break;
        }
    }

    debug!("Monitoring in {} mode", *mode);
    match *mode {
        StopOnFailure => {
            while monitor.ok() {
                match monitor.next_event() {
                    Exit(cname, pid, status) => {
                        error!("Process {}:{} dead with status {}. Failing.",
                            cname, pid, status);
                        monitor.send_all(SIGTERM);
                        monitor.fail();
                    }
                    Signal(sig)
                    if sig == SIGTERM || sig == SIGINT || sig == SIGQUIT => {
                        debug!("Got {}. Propagating.", sig);
                        monitor.send_all(sig);
                        monitor.fail();
                    }
                    Signal(sig) => {
                        debug!("Got {}. Ignoring.", sig);
                    }
                }
            }
        }
        Restart => {
            while monitor.ok() {
                match monitor.next_event() {
                    Exit(cname, pid, status) => {
                        error!("Process {}:{} dead with status {}",
                            cname, pid, status);
                        start(&cname, &mut monitor);
                    }
                    Signal(sig)
                    if sig == SIGTERM || sig == SIGINT || sig == SIGQUIT => {
                        debug!("Got {}. Propagating.", sig);
                        monitor.send_all(sig);
                        monitor.fail();
                    }
                    Signal(sig) => {
                        debug!("Got {}. Ignoring.", sig);
                    }
                }
            }
        }
        WaitAll => {},
    }

    // Wait all mode always at the end
    debug!("Falled back to WaitAll mode");
    monitor.wait_all();

    return Ok(monitor.get_status());
}

pub fn exec_supervise_command(_env: &Environ, _workdir: &Path,
    _command: &Command, _container: &Container)
    -> Result<pid_t, String> {
    fail!("Nested supervise commands do not work yet");
}
