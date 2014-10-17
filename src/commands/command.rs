use std::io::stdio::{stdout, stderr};
use std::io::timer::sleep;
use std::os::getenv;
use std::default::Default;
use std::time::duration::Duration;


use libc::pid_t;
use collections::treemap::TreeMap;
use argparse::{ArgumentParser, StoreFalse};

use super::super::env::{Environ, Container};
use super::super::options::env_options;
use super::super::run::internal_run;
use super::super::config::{Plain, Command};
use super::super::build::ensure_container;
use super::super::monitor::Monitor;
use super::super::utils::run::{write_sentinel, check_command_workdir};
use super::super::utils::run::{is_writeable, print_banner, container_volumes};
use super::super::linux::{RunOptions};


pub fn run_plain_command(env: &mut Environ, cmdname: &String,
    args: Vec<String>)
    -> Result<int, String>
{
    let has_arguments;
    let description;
    let mut command_workdir;
    {
        let command = env.config.commands.find(cmdname).unwrap();
        has_arguments = command.accepts_arguments;
        description = command.description.clone().unwrap_or("".to_string());
        command_workdir = command.work_dir.is_some();
    }
    let mut cmdargs;
    if has_arguments {
        //  All options forwarded to command (including --help and others)
        cmdargs = args;
        cmdargs.shift();  // Zeroth arg is a command
    } else {
        //  We can provide useful help in this case
        cmdargs = Vec::new();
        let mut ap = ArgumentParser::new();
        if description.len() > 0 {
            ap.set_description(description.as_slice());
        }
        if command_workdir {
            ap.refer(&mut command_workdir)
                .add_option(["--override-workdir"], box StoreFalse,
                    "Do not obey `work-dir` parameter in command definition.
                     Use current working directory instead.");
        }
        env_options(env, &mut ap);
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => return Ok(122),
        }
    }
    let command = env.config.commands.find(cmdname).unwrap();
    let cname = env.container.as_ref().or(command.container.as_ref()).unwrap();
    let mut container = try!(env.get_container(cname));
    try!(ensure_container(env, &mut container));

    let _sent = try!(write_sentinel(env, &mut container, command.write_mode));
    let work_dir = try!(check_command_workdir(env, command));

    if command.banner_delay == 0 {
        print_banner(&command.banner);
    }
    let mut monitor = Monitor::new(true);
    let pid = try!(exec_plain_command_args(env, &work_dir,
        command, &container, cmdargs));
    monitor.add("child".to_string(), pid);
    if command.banner_delay > 0 {
        sleep(Duration::seconds(command.banner_delay));
        print_banner(&command.banner);
    }
    monitor.wait_all();
    let result = monitor.get_status();
    if result == 0 {
        print_banner(&command.epilog);
    }
    return Ok(result);
}

pub fn exec_plain_command(env: &Environ, work_dir: &Path,
    command: &Command, container: &Container)
    -> Result<pid_t, String>
{
    return exec_plain_command_args(env, work_dir,
        command, container, Vec::new());
}

pub fn exec_plain_command_args(env: &Environ, work_dir: &Path,
    command: &Command, container: &Container, cmdargs: Vec<String>)
    -> Result<pid_t, String>
{
    let mut runenv = TreeMap::new();
    for (k, v) in command.environ.iter() {
        runenv.insert(k.clone(), v.clone());
    }
    for k in command.inherit_environ.iter() {
        match getenv(k.as_slice()) {
            Some(ref val) => { runenv.insert(k.clone(), val.clone()); }
            None => {}
        }
    }
    let mut argprefix: Vec<String> = Vec::new();
    match container.command_wrapper {
        Some(ref wrapper) => {
            argprefix.extend(wrapper.clone().move_iter());
        }
        None => {}
    }
    match command.execute {
        Plain(ref cmd) => argprefix.push_all(cmd.as_slice()),
        _ => unreachable!(),
    }
    let cmd = argprefix.shift().unwrap();
    let ropts = RunOptions {
        pid1mode: command.pid1mode,
        writeable: is_writeable(command.write_mode),
        mounts: container_volumes(env, container),
        .. Default::default()
        };
    return internal_run(env, container, ropts, command.resolv_conf,
        work_dir, cmd, (argprefix + cmdargs), runenv);
}
