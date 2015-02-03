use std::io::fs::PathExtensions;

use super::super::context::BuildContext;
use container::monitor::{Monitor};
use container::monitor::MonitorResult::{Exit, Killed};
use container::container::{Command};


fn find_cmd(ctx: &mut BuildContext, cmd: &str) -> Result<Path, String> {
    let rpath = Path::new("/");
    if let Some(paths) = ctx.environ.get(&"PATH".to_string()) {
        for dir in paths.as_slice().split(':') {
            let path = Path::new(dir);
            if !path.is_absolute() {
                warn!("All items in PATH must be absolute, not {}",
                      path.display());
                continue;
            }
            if path.path_relative_from(&rpath).unwrap().join(cmd).exists() {
                return Ok(path.join(cmd));
            }
        }
    }
    return Err(format!("Command {} not found", cmd));
}

pub fn run_command_at(ctx: &mut BuildContext, cmdline: &[String], path: &Path)
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

pub fn run_command(ctx: &mut BuildContext, cmd: &[String])
    -> Result<(), String>
{
    return run_command_at(ctx, cmd, &Path::new("/work"));
}
