use std::rc::Rc;
use std::os::{getenv};
use std::os::self_exe_path;
use std::io::stdio::{stdout, stderr};
use std::collections::TreeSet;

use argparse::{ArgumentParser};

use container::monitor::{Monitor, RunOnce, Exit, Killed};
use container::container::{Command};
use config::Config;
use config::command::main;
use config::command::{SuperviseInfo, stop_on_failure};



pub fn run_user_command(config: &Config, workdir: &Path,
    cmd: String, args: Vec<String>)
    -> Result<int, String>
{
    if cmd.as_slice() == "_run" {
        return run_simple_command(workdir, cmd, args);
    }
    match config.commands.find(&cmd) {
        None => Err(format!("Command {} not found. \
                    Run vagga without arguments to see the list.", cmd)),
        Some(&main::Command(_)) => run_simple_command(workdir, cmd, args),
        Some(&main::Supervise(ref sup))
        => run_supervise_command(config, workdir, sup, cmd, args),
    }
}

fn _common(cmd: &mut Command, workdir: &Path) {
    cmd.set_env("TERM".to_string(),
                getenv("TERM").unwrap_or("dumb".to_string()));
    if let Some(x) = getenv("RUST_LOG") {
        cmd.set_env("RUST_LOG".to_string(), x);
    }
    if let Some(x) = getenv("RUST_BACKTRACE") {
        cmd.set_env("RUST_BACKTRACE".to_string(), x);
    }
    if let Some(x) = getenv("HOME") {
        cmd.set_env("VAGGA_USER_HOME".to_string(), x);
    }
    cmd.set_env("PWD".to_string(), Path::new("/work")
        .join(workdir)
        .display().to_string());
}

fn run_simple_command(workdir: &Path, cmdname: String, args: Vec<String>)
    -> Result<int, String>
{
    let mut cmd = Command::new("wrapper".to_string(),
        self_exe_path().unwrap().join("vagga_wrapper"));
    cmd.keep_sigmask();
    cmd.arg(cmdname.as_slice());
    cmd.args(args.as_slice());
    _common(&mut cmd, workdir);
    cmd.container();
    cmd.set_max_uidmap();
    match Monitor::run_command(cmd) {
        Killed => Ok(143),
        Exit(val) => Ok(val),
    }
}

fn run_supervise_command(_config: &Config, workdir: &Path,
    sup: &SuperviseInfo, cmdname: String, mut args: Vec<String>)
    -> Result<int, String>
{
    if sup.mode != stop_on_failure {
        fail!("Only stop-on-failure mode implemented");
    }
    {
        args.insert(0, "vagga ".to_string() + cmdname);
        let mut ap = ArgumentParser::new();
        ap.set_description(sup.description.as_ref().map(|x| x.as_slice())
            .unwrap_or("Run multiple processes simultaneously"));
        // TODO(tailhook) implement --only and --exclude
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => {
                return Ok(122);
            }
        }
    }
    let mut containers = TreeSet::new();
    for (_, child) in sup.children.iter() {
        let cont = child.get_container();
        if !containers.contains(cont) {
            containers.insert(cont.to_string());
            match run_simple_command(workdir,
                "_build".to_string(), vec!(cont.to_string()))
            {
                Ok(0) => {}
                x => return x,
            }
        }
    }
    let mut mon = Monitor::new();
    for (name, _) in sup.children.iter() {
        let mut cmd = Command::new("wrapper".to_string(),
            self_exe_path().unwrap().join("vagga_wrapper"));
        cmd.keep_sigmask();
        cmd.arg(cmdname.as_slice());
        cmd.arg(name.as_slice());
        _common(&mut cmd, workdir);
        cmd.container();
        cmd.set_max_uidmap();
        mon.add(Rc::new(name.clone()), box RunOnce::new(cmd));
    }
    match mon.run() {
        Killed => Ok(143),
        Exit(val) => Ok(val),
    }
}
