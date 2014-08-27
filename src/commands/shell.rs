use std::io::stdio::{stdout, stderr};
use std::os::getenv;
use std::default::Default;

use libc::pid_t;
use collections::treemap::TreeMap;
use argparse::{ArgumentParser, StoreFalse};

use super::super::env::{Environ, Container};
use super::super::options::env_options;
use super::super::run::internal_run;
use super::super::config::{Shell, Command};
use super::super::build::ensure_container;
use super::super::monitor::Monitor;
use super::super::utils::run::{write_sentinel, check_command_workdir};
use super::super::utils::run::{is_writeable};
use super::super::linux::RunOptions;


pub fn run_shell_command(env: &mut Environ, cmdname: &String,
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
                .add_option(["--override-work-dir"], box StoreFalse,
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

    let mut monitor = Monitor::new(true);
    let pid = try!(exec_shell_command_args(env, &work_dir,
        command, &container, cmdargs));
    monitor.add("child".to_string(), pid);
    monitor.wait_all();
    return Ok(monitor.get_status());
}

pub fn exec_shell_command(env: &Environ, work_dir: &Path,
    command: &Command, container: &Container)
    -> Result<pid_t, String>
{
    return exec_shell_command_args(env, work_dir,
        command, container, Vec::new());
}

pub fn exec_shell_command_args(env: &Environ, work_dir: &Path,
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
    argprefix.extend(container.shell.clone().move_iter());
    match command.execute {
        Shell(ref x) => argprefix.push(x.clone()),
        _ => unreachable!(),
    }
    let cmd = argprefix.shift().unwrap();
    let ropts = RunOptions {
        pid1mode: command.pid1mode,
        writeable: is_writeable(command.write_mode),
        .. Default::default()
        };
    return internal_run(env, container, ropts, command.resolv_conf,
        work_dir, cmd, (argprefix + cmdargs), runenv);
}
