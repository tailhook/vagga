use std::path::Path;

use unshare::{Command};

use options::build_mode::BuildMode;
use config::{Config, Settings};
use config::command::MainCommand;
use config::command::{CommandInfo, Networking, WriteMode};

use super::supervisor;
use super::build::{build_container};
use super::wrap::Wrapper;


pub fn run_user_command(config: &Config, settings: &Settings, workdir: &Path,
    cmd: String, args: Vec<String>, build_mode: BuildMode)
    -> Result<i32, String>
{
    match config.commands.get(&cmd) {
        None => Err(format!("Command {} not found. \
                    Run vagga without arguments to see the list.", cmd)),
        Some(&MainCommand::Command(ref info))
        => run_simple_command(settings, info, workdir, cmd, args, build_mode),
        Some(&MainCommand::Supervise(ref sup))
        => supervisor::run_supervise_command(settings, workdir, sup,
            cmd, args, build_mode),
    }
}

pub fn run_simple_command(settings: &Settings, cfg: &CommandInfo,
    workdir: &Path, cmdname: String, args: Vec<String>,
    build_mode: BuildMode)
    -> Result<i32, String>
{
    if let Some(_) = cfg.network {
        return Err(format!(
            "Network is not supported for !Command use !Supervise"))
    }
    let ver = try!(build_container(settings, &cfg.container, build_mode));
    let mut cmd: Command = Wrapper::new(Some(&ver), settings);
    cmd.workdir(workdir);
    cmd.arg(cmdname);
    cmd.args(&args);
    if cfg.network.is_none() {
        cmd.userns();
    }
    let res = cmd.run();

    if cfg.write_mode != WriteMode::read_only {
        let mut cmd: Command = Wrapper::new(None, settings);
        cmd.workdir(workdir);
        cmd.userns();
        cmd.arg("_clean").arg("--transient");
        match cmd.run() {
            Ok(0) => {}
            x => warn!(
                "The `vagga _clean --transient` exited with status: {:?}", x),
        }

    }
    if res == Ok(0) {
        if let Some(ref epilog) = cfg.epilog {
            print!("{}", epilog);
        }
    }
    return res;
}
