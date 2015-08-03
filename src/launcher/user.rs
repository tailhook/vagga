use std::env;
use std::path::Path;

use container::monitor::{Monitor};
use container::monitor::MonitorResult::{Exit, Killed};
use container::container::{Command};
use config::Config;
use config::command::MainCommand;
use config::command::{CommandInfo, Networking, WriteMode};

use super::supervisor;
use super::build::build_container;


pub fn run_user_command(config: &Config, workdir: &Path,
    cmd: String, args: Vec<String>)
    -> Result<i32, String>
{
    match config.commands.get(&cmd) {
        None => Err(format!("Command {} not found. \
                    Run vagga without arguments to see the list.", cmd)),
        Some(&MainCommand::Command(ref info))
        => run_simple_command(config, info, workdir, cmd, args),
        Some(&MainCommand::Supervise(ref sup))
        => supervisor::run_supervise_command(config, workdir, sup, cmd, args),
    }
}

pub fn common_child_command_env(cmd: &mut Command, workdir: Option<&Path>) {
    for (k, v) in env::vars() {
        if k.starts_with("VAGGAENV_") {
            cmd.set_env(k, v);
        }
    }
    cmd.set_env("TERM".to_string(),
                env::var("TERM").unwrap_or("dumb".to_string()));
    if let Ok(x) = env::var("PATH") {
        cmd.set_env("HOST_PATH".to_string(), x);
    }
    if let Ok(x) = env::var("RUST_LOG") {
        cmd.set_env("RUST_LOG".to_string(), x);
    }
    if let Ok(x) = env::var("RUST_BACKTRACE") {
        cmd.set_env("RUST_BACKTRACE".to_string(), x);
    }
    if let Ok(x) = env::var("HOME") {
        cmd.set_env("VAGGA_USER_HOME".to_string(), x);
    }
    if let Some(x) = workdir {
        cmd.set_env("PWD".to_string(),
            Path::new("/work").join(x).display().to_string());
    }
}

pub fn run_simple_command(config: &Config, cfg: &CommandInfo,
    workdir: &Path, cmdname: String, args: Vec<String>)
    -> Result<i32, String>
{
    if let Some(_) = cfg.network {
        return Err(format!(
            "Network is not supported for !Command use !Supervise"))
    }
    try!(build_container(config, &cfg.container));
    let res = run_wrapper(Some(workdir), cmdname, args, cfg.network.is_none());

    if cfg.write_mode != WriteMode::read_only {
        match run_wrapper(Some(workdir), "_clean".to_string(),
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
pub fn run_wrapper(workdir: Option<&Path>, cmdname: String, args: Vec<String>,
    userns: bool)
    -> Result<i32, String>
{
    let mut cmd = Command::vagga("vagga_wrapper");
    cmd.keep_sigmask();
    cmd.arg(&cmdname);
    cmd.args(&args);
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

