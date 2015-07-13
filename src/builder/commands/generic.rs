use std::cell::RefCell;
use std::rc::Rc;
use std::path::{Path, PathBuf};

use nix::unistd::{pipe, close};

use super::super::context::BuildContext;
use container::monitor::{Monitor, Executor, MonitorStatus};
use container::monitor::MonitorResult::{Exit, Killed};
use container::container::{Command};


fn find_cmd(ctx: &mut BuildContext, cmd: &str) -> Result<PathBuf, String> {
    let rpath = Path::new("/");
    let chroot = Path::new("/vagga/root");
    if let Some(paths) = ctx.environ.get(&"PATH".to_string()) {
        for dir in paths.as_slice().split(':') {
            let path = Path::new(dir);
            if !path.is_absolute() {
                warn!("All items in PATH must be absolute, not {}",
                      path.display());
                continue;
            }
            if chroot.join(path.path_relative_from(&rpath).unwrap())
                .join(cmd).exists()
            {
                return Ok(path.join(cmd));
            }
        }
        return Err(format!("Command {:?} not found in {:?}", cmd, paths));
    }
    return Err(format!("Command {:?} not found (no PATH)", cmd));
}

pub fn run_command_at_env(ctx: &mut BuildContext, cmdline: &[String],
    path: &Path, env: &[(&str, &str)])
    -> Result<(), String>
{
    let cmdpath = if cmdline[0].as_slice().starts_with("/") {
        Path::new(cmdline[0].as_slice())
    } else {
        try!(find_cmd(ctx, cmdline[0].as_slice()))
    };

    let mut cmd = Command::new("run".to_string(), &cmdpath);
    cmd.set_workdir(path);
    cmd.chroot(&Path::new("/vagga/root"));
    cmd.args(cmdline[1..].as_slice());
    for (k, v) in ctx.environ.iter() {
        cmd.set_env(k.clone(), v.clone());
    }
    for &(k, v) in env.iter() {
        cmd.set_env(k.to_string(), v.to_string());
    }

    debug!("Running {:?}", cmd);

    match Monitor::run_command(cmd) {
        Killed => {
            return Err(format!("Command {:?} is dead", cmdline));
        }
        Exit(0) => {
            return Ok(())
        }
        Exit(val) => {
            return Err(format!("Command {:?} exited with status {}",
                               cmdline, val));
        }
    }
}

pub fn run_command_at(ctx: &mut BuildContext, cmdline: &[String], path: &Path)
    -> Result<(), String>
{
    run_command_at_env(ctx, cmdline, path, &[])
}

pub fn run_command(ctx: &mut BuildContext, cmd: &[String])
    -> Result<(), String>
{
    return run_command_at_env(ctx, cmd, &Path::new("/work"), &[]);
}

pub fn capture_command<'x>(ctx: &mut BuildContext, cmdline: &'x[String],
    env: &[(&str, &str)])
    -> Result<Vec<u8>, String>
{
    let cmdpath = if cmdline[0].as_slice().starts_with("/") {
        Path::new(cmdline[0].as_slice())
    } else {
        try!(find_cmd(ctx, cmdline[0].as_slice()))
    };

    let mut cmd = Command::new("run".to_string(), &cmdpath);
    cmd.chroot(&Path::new("/vagga/root"));
    cmd.args(cmdline[1..].as_slice());
    for (k, v) in ctx.environ.iter() {
        cmd.set_env(k.clone(), v.clone());
    }
    for &(k, v) in env.iter() {
        cmd.set_env(k.to_string(), v.to_string());
    }
    debug!("Running {:?}", cmd);
    let (res, data) = unsafe {
        let pipe = try!(pipe()
            .map_err(|e| format!("Can't create pipe: {}", e)));
        cmd.set_stdout_fd(pipe.writer);
        let res = Monitor::run_command(cmd);
        (res, pipe.read())
    };

    match res {
        Killed => {
            return Err(format!("Command {:?} is dead", cmdline));
        }
        Exit(0) => {
            return Ok(data);
        }
        Exit(val) => {
            return Err(format!("Command {:?} exited with status {}",
                               cmdline, val));
        }
    }
}
