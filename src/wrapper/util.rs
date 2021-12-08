use std::path::{Path, PathBuf};
use std::collections::BTreeMap;

use unshare::{Command};

use crate::config::Container;
use crate::config::command::Run;


pub fn find_cmd(cmd: &str, env: &BTreeMap<String, String>)
    -> Result<PathBuf, String>
{
    if cmd.contains("/") {
        return Ok(PathBuf::from(cmd));
    } else {
        if let Some(paths) = env.get(&"PATH".to_string()) {
            for dir in paths[..].split(':') {
                let path = Path::new(dir);
                if !path.is_absolute() {
                    warn!("All items in PATH must be absolute, not {:?}",
                          path);
                    continue;
                }
                let path = path.join(cmd);
                if path.exists() {
                    return Ok(path);
                }
            }
            return Err(format!("Command {} not found in {:?}",
                cmd, paths));
        } else {
            return Err(format!("Command {} is not absolute and no PATH set",
                cmd));
        }
    }
}

pub fn warn_if_data_container(container_config: &Container) {
    if container_config.is_data_container() {
        warn!("You are trying to run command inside the data container. \
            Data containers is designed to use as volumes inside other \
            containers. Usually there are no system dirs at all.");
    }
}

pub fn gen_command(default_shell: &Vec<String>, cmdline: &Run,
                   env: &BTreeMap<String, String>)
    -> Result<Command, String>
{
    match *cmdline {
        Run::Shell(ref data) => {
            if default_shell.len() > 0 {
                let mut cmd = Command::new(&default_shell[0]);
                for arg in &default_shell[1..] {
                    if arg == "$cmdline" {
                        cmd.arg(data);
                    } else {
                        cmd.arg(arg);
                    }
                }
                return Ok(cmd);
            } else {
                let mut cmd = Command::new("/bin/sh");
                cmd.arg("-c");
                cmd.arg(data);
                cmd.arg("--");
                return Ok(cmd);
            }
        }
        Run::Command(ref cmdline) => {
            let cpath = find_cmd(&cmdline[0], &env)?;
            let mut cmd = Command::new(&cpath);
            cmd.args(&cmdline[1..]);
            return Ok(cmd);
        }
    }
}
