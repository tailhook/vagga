use std::os::{getenv};
use std::os::self_exe_path;

use container::monitor::{Monitor};
use container::monitor::MonitorResult::{Exit, Killed};
use container::container::{Command};
use config::Config;
use config::command::MainCommand;
use config::command::{CommandInfo, Networking, WriteMode};

use super::supervisor;


pub fn run_user_command(config: &Config, workdir: &Path,
    cmd: String, args: Vec<String>)
    -> Result<isize, String>
{
    match config.commands.get(&cmd) {
        None => Err(format!("Command {} not found. \
                    Run vagga without arguments to see the list.", cmd)),
        Some(&MainCommand::Command(ref info))
        => run_simple_command(info, workdir, cmd, args),
        Some(&MainCommand::Supervise(ref sup))
        => supervisor::run_supervise_command(config, workdir, sup, cmd, args),
    }
}

pub fn common_child_command_env(cmd: &mut Command, workdir: &Path) {
    cmd.set_env("TERM".to_string(),
                getenv("TERM").unwrap_or("dumb".to_string()));
    if let Some(x) = getenv("PATH") {
        cmd.set_env("HOST_PATH".to_string(), x);
    }
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

pub fn run_simple_command(cfg: &CommandInfo,
    workdir: &Path, cmdname: String, args: Vec<String>)
    -> Result<isize, String>
{
    if let Some(_) = cfg.network {
        return Err(format!(
            "Network is not supported for !Command use !Supervise"))
    }
    match run_wrapper(workdir, "_build".to_string(),
        vec!(cfg.container.clone()), true)
    {
        Ok(0) => {}
        x => return x,
    }
    let res = run_wrapper(workdir, cmdname, args, cfg.network.is_none());

    if cfg.write_mode != WriteMode::read_only {
        match run_wrapper(workdir, "_clean".to_string(),
            vec!("--transient".to_string()), true)
        {
            Ok(0) => {}
            x => warn!(
                "The `vagga _clean --transient` exited with status: {:?}", x),
        }

    }
    return res;
}

// TODO(tailhook) run not only for simple commands
pub fn run_wrapper(workdir: &Path, cmdname: String, args: Vec<String>,
    userns: bool)
    -> Result<isize, String>
{
    let mut cmd = Command::new("wrapper".to_string(),
        self_exe_path().unwrap().join("vagga_wrapper"));
    cmd.keep_sigmask();
    cmd.arg(cmdname.as_slice());
    cmd.args(args.as_slice());
    common_child_command_env(&mut cmd, workdir);
    cmd.container();
    if userns {
        cmd.set_max_uidmap();
    }
    match Monitor::run_command(cmd) {
        Killed => Ok(143),
        Exit(val) => Ok(val),
    }
}

