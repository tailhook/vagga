use std::path::Path;

use unshare::{Command};

use options::build_mode::BuildMode;
use config::{Config, Settings};
use config::command::MainCommand;
use config::command::{CommandInfo, Networking, WriteMode};
use process_util::{run_and_wait, convert_status};
use super::supervisor;
use super::build::{build_container};
use super::wrap::Wrapper;
use launcher::volumes::prepare_volumes;


pub fn run_user_command(config: &Config, settings: &Settings, workdir: &Path,
    cmd: String, args: Vec<String>, build_mode: BuildMode)
    -> Result<i32, String>
{
    match config.commands.get(&cmd) {
        None => Err(format!("Command {} not found. \
                    Run vagga without arguments to see the list.", cmd)),
        Some(&MainCommand::Command(ref info))
        => run_simple_command(config, settings, info, workdir,
                              cmd, args, build_mode),
        Some(&MainCommand::Supervise(ref sup))
        => supervisor::run_supervise_command(config, settings, workdir, sup,
            cmd, args, build_mode),
    }
}

pub fn run_simple_command(config: &Config, settings: &Settings,
    cinfo: &CommandInfo,
    workdir: &Path, cmdname: String, args: Vec<String>,
    build_mode: BuildMode)
    -> Result<i32, String>
{
    if let Some(_) = cinfo.network {
        return Err(format!(
            "Network is not supported for !Command use !Supervise"))
    }
    let ver = try!(build_container(settings, &cinfo.container, build_mode));
    let cont = try!(config.containers.get(&cinfo.container)
        .ok_or_else(|| format!("Container {:?} not found", cinfo.container)));
    try!(prepare_volumes(cont.volumes.values(), settings, build_mode));
    try!(prepare_volumes(cinfo.volumes.values(), settings, build_mode));
    let mut cmd: Command = Wrapper::new(Some(&ver), settings);
    cmd.workdir(workdir);
    cmd.arg(cmdname);
    cmd.args(&args);
    if cinfo.network.is_none() {
        cmd.userns();
    }
    let res = run_and_wait(&mut cmd).map(convert_status);

    if cinfo.write_mode != WriteMode::read_only {
        let mut cmd: Command = Wrapper::new(None, settings);
        cmd.workdir(workdir);
        cmd.userns();
        cmd.arg("_clean").arg("--transient");
        match cmd.status() {
            Ok(s) if s.success() => {}
            Ok(s) => warn!("The `vagga _clean --transient` {}", s),
            Err(e) => warn!("Failed to run `vagga _clean --transient`: {}", e),
        }

    }
    if res == Ok(0) {
        if let Some(ref epilog) = cinfo.epilog {
            print!("{}", epilog);
        }
    }
    return res;
}
