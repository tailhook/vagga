use std::io::stdio::{stdout, stderr};
use std::io::fs::readlink;
use std::io::signal::Listener;
use Sig = std::io::signal;

use collections::treemap::TreeMap;
use argparse::{ArgumentParser, List};

use super::super::env::{Environ, Container};
use super::super::options::env_options;
use super::super::build::{build_container, link_container};
use super::super::config::{Shell, Plain, Supervise};
use super::super::config::Command;
use super::super::commands::shell::exec_shell_command;
use super::super::commands::command::exec_plain_command;


pub fn run_supervise_command(env: &mut Environ, cmdname: &String,
    args: Vec<String>)
    -> Result<int, String>
{
    let mut processes: Vec<String> = Vec::new();
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut processes)
            .add_argument("subrocess", box List::<String>,
                "A subset of processes to run. All will be run by default");
        env_options(env, &mut ap);
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => return Ok(122),
        }
    }
    let command = env.config.commands.find(cmdname).unwrap();
    let (mode, processes) = match command.execute {
        Supervise(ref mode, ref processes) => (mode, processes),
        _ => unreachable!(),
    };
    let mut containers = TreeMap::new();
    for (_, command) in processes.iter() {
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
        containers.insert(cname.clone(), container);
    }

    let mut sig = Listener::new();
    try!(sig.register(Sig::Interrupt).map_err(
        |e| format!("Can't listen SIGINT: {}", e)));
    try!(sig.register(Sig::Quit).map_err(
        |e| format!("Can't listen SIGQUIT: {}", e)));
    try!(sig.register(Sig::HangUp).map_err(
        |e| format!("Can't listen SIGHUP: {}", e)));

    for (_, command) in processes.iter() {
        let fun = match command.execute {
            Shell(_) => exec_shell_command,
            Plain(_) => exec_plain_command,
            Supervise(_, _) => exec_supervise_command,
        };
        let container = containers.find(command.container.as_ref().unwrap());
        fun(env, command, container.unwrap());
    }

    return Ok(0);
}

pub fn exec_supervise_command(env: &Environ, command: &Command,
    container: &Container)
    -> Result<int, String> {
    fail!("Nested supervise commands do not work yet");
}
