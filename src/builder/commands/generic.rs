use std::path::{Path, PathBuf};

use unshare::{Command};

use super::super::context::BuildContext;
use super::super::super::path_util::ToRelative;
use path_util::PathExt;
use process_util::capture_stdout;


fn find_cmd(ctx: &mut BuildContext, cmd: &str) -> Result<PathBuf, String> {
    let chroot = Path::new("/vagga/root");
    if let Some(paths) = ctx.environ.get(&"PATH".to_string()) {
        for dir in paths[..].split(':') {
            let path = Path::new(dir);
            if !path.is_absolute() {
                warn!("All items in PATH must be absolute, not {}",
                      path.display());
                continue;
            }
            if chroot.join(path.rel())
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
    let cmdpath = if cmdline[0][..].starts_with("/") {
        PathBuf::from(&cmdline[0])
    } else {
        try!(find_cmd(ctx, &cmdline[0]))
    };

    let mut cmd = Command::new(&cmdpath);
    cmd.current_dir(path);
    cmd.chroot_dir("/vagga/root");
    cmd.args(&cmdline[1..]);
    cmd.env_clear();
    for (k, v) in ctx.environ.iter() {
        cmd.env(k, v);
    }
    for &(k, v) in env.iter() {
        cmd.env(k, v);
    }

    debug!("Running {:?}", cmd);

    match cmd.status() {
        Ok(ref s) if s.success() => {
            return Ok(());
        }
        Ok(s) => {
            return Err(format!("Command {:?} {}", cmd, s));
        }
        Err(e) => {
            return Err(format!("Couldn't run {:?}: {}", cmd, e));
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
    let cmdpath = if cmdline[0][..].starts_with("/") {
        PathBuf::from(&cmdline[0])
    } else {
        try!(find_cmd(ctx, &cmdline[0]))
    };

    let mut cmd = Command::new(&cmdpath);
    cmd.chroot_dir("/vagga/root");
    cmd.args(&cmdline[1..]);
    cmd.env_clear();
    for (k, v) in ctx.environ.iter() {
        cmd.env(k, v);
    }
    for &(k, v) in env.iter() {
        cmd.env(k, v);
    }
    capture_stdout(cmd)
}
