use std::env;
use std::path::Path;

use unshare::{Command, Namespace};

use options;
use config::Config;
use config::command::MainCommand;
use config::command::{CommandInfo, Networking, WriteMode};

use super::supervisor;
use super::build::{build_container};
use process_util::{convert_status, set_uidmap};
use container::uidmap::get_max_uidmap;


pub fn run_user_command(config: &Config, workdir: &Path,
    cmd: String, args: Vec<String>, build_mode: options::BuildMode)
    -> Result<i32, String>
{
    match config.commands.get(&cmd) {
        None => Err(format!("Command {} not found. \
                    Run vagga without arguments to see the list.", cmd)),
        Some(&MainCommand::Command(ref info))
        => run_simple_command(config, info, workdir, cmd, args, build_mode),
        Some(&MainCommand::Supervise(ref sup))
        => supervisor::run_supervise_command(config, workdir, sup,
            cmd, args, build_mode),
    }
}

pub fn common_child_command_env(cmd: &mut Command, workdir: Option<&Path>) {
    for (k, v) in env::vars() {
        if k.starts_with("VAGGAENV_") {
            cmd.env(k, v);
        }
    }
    cmd.env("TERM", env::var("TERM").unwrap_or("dumb".to_string()));
    if let Ok(x) = env::var("PATH") {
        cmd.env("HOST_PATH", x);
    }
    if let Ok(x) = env::var("RUST_LOG") {
        cmd.env("RUST_LOG", x);
    }
    if let Ok(x) = env::var("RUST_BACKTRACE") {
        cmd.env("RUST_BACKTRACE", x);
    }
    if let Ok(x) = env::var("HOME") {
        cmd.env("VAGGA_USER_HOME", x);
    }
    if let Some(x) = workdir {
        cmd.env("PWD", Path::new("/work").join(x));
    }
}

pub fn run_simple_command(_config: &Config, cfg: &CommandInfo,
    workdir: &Path, cmdname: String, args: Vec<String>,
    build_mode: options::BuildMode)
    -> Result<i32, String>
{
    if let Some(_) = cfg.network {
        return Err(format!(
            "Network is not supported for !Command use !Supervise"))
    }
    let ver = try!(build_container(&cfg.container, build_mode));
    let res = run_wrapper(Some(workdir), cmdname, args,
        cfg.network.is_none(), Some(&ver));

    if cfg.write_mode != WriteMode::read_only {
        match run_wrapper(Some(workdir), "_clean".to_string(),
            vec!("--transient".to_string()), true, None)
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
    userns: bool, root: Option<&str>)
    -> Result<i32, String>
{
    let mut cmd = Command::new("/proc/self/exe");
    cmd.arg0("vagga_wrapper");
    if let Some(root) = root {
        cmd.arg("--root");
        cmd.arg(root);
    };
    cmd.arg(&cmdname);
    cmd.args(&args);
    cmd.env_clear();
    common_child_command_env(&mut cmd, workdir);
    cmd.unshare(
        [Namespace::Mount, Namespace::Ipc, Namespace::Pid].iter().cloned());
    if userns {
        set_uidmap(&mut cmd, &get_max_uidmap().unwrap(), true);
    }
    match cmd.status() {
        Ok(x) => Ok(convert_status(x)),
        Err(e) => Err(format!("Error running {:?}: {}", cmd, e)),
    }
}

