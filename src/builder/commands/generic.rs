use std::rc::Rc;
use std::io::fs::PathExtensions;

use super::super::context::BuildContext;
use container::monitor::{Monitor, Exit, Killed, Executor};
use container::container::{Command};

struct RunCommand<'a> {
    work_dir: &'a Path,
    cmd: Path,
    args: &'a [String],
    ctx: &'a BuildContext<'a>,
}

impl<'a> Executor for RunCommand<'a> {
    fn command(&self) -> Command {
        let mut cmd = Command::new("run".to_string(), &self.cmd);
        cmd.set_workdir(self.work_dir);
        cmd.chroot(&Path::new("/vagga/root"));
        cmd.args(self.args.as_slice());
        for (k, v) in self.ctx.environ.iter() {
            cmd.set_env(k.clone(), v.clone());
        }
        return cmd;
    }
}

fn find_cmd(ctx: &mut BuildContext, cmd: &str) -> Result<Path, String> {
    let rpath = Path::new("/");
    if let Some(paths) = ctx.environ.find(&"PATH".to_string()) {
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

pub fn run_command_at(ctx: &mut BuildContext, cmd: &[String], path: &Path)
    -> Result<(), String>
{
    let mut mon = Monitor::new();

    let cmdpath = if cmd[0].as_slice().starts_with("/") {
        Path::new(cmd[0].as_slice())
    } else {
        try!(find_cmd(ctx, cmd[0].as_slice()))
    };

    mon.add(Rc::new("command".to_string()), box RunCommand {
        cmd: cmdpath,
        work_dir: path,
        args: cmd[1..],
        ctx: ctx,
    });
    match mon.run() {
        Killed => {
            return Err(format!("Command {} is dead", cmd));
        }
        Exit(0) => {
            return Ok(())
        }
        Exit(val) => {
            return Err(format!("Command {} exited with status {}", cmd, val));
        }
    }
}

pub fn run_command(ctx: &mut BuildContext, cmd: &[String])
    -> Result<(), String>
{
    return run_command_at(ctx, cmd, &Path::new("/work"));
}
