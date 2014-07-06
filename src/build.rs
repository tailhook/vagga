use std::io;
use std::os::getenv;
use std::io::fs::{mkdir, rmdir_recursive, rename};
use std::io::process::{ExitStatus, Command, Ignored, InheritFd};

use super::env::Environ;
use super::config::Config;

pub struct BuildTask<'a> {
    pub environ: &'a Environ,
    pub config: &'a Config,
    pub name: &'a String,
    pub work_dir: &'a Path,
    pub project_root: &'a Path,
    pub stderr: &'a mut Writer,
}

fn makedirs(path: &Path) -> Result<(),String> {
    if path.exists() {
        return Ok(());
    }
    try!(makedirs(&path.dir_path()));
    return match mkdir(path, io::UserRWX) {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("Can't mkdir: {}", e)),
    };
}

pub fn build_container(task: BuildTask) -> Result<(),String>
{
    let container = match task.config.containers.find(task.name) {
        Some(c) => c,
        None => {
            return Err(format!("Can't find container {} in config", task.name));
        }
    };
    task.stderr.write_line(format!("Building {}", task.name).as_slice()).ok();

    let builder = &container.builder;
    let bexe = task.environ.vagga_dir.join_many(
        ["build-scripts", builder.as_slice()]);
    if !bexe.exists() {
        return Err(format!("Builder {} does not exist", builder));
    }

    task.stderr.write_line(format!(
        "Builder {}", bexe.display()).as_slice()).ok();

    let mut env = Vec::new();
    let container_dir = task.project_root
        .join_many([".vagga", task.name.as_slice()]);
    let container_root = container_dir.join("root");
    let container_tmp = container_dir.join(".tmproot");

    if container_tmp.exists() {
        match rmdir_recursive(&container_tmp) {
            Ok(()) => {}
            Err(x) => return Err(format!("Can't clean temporary root: {}", x)),
        }
    }
    try!(makedirs(&container_tmp));

    env.push(("PATH".to_string(), getenv("PATH").unwrap()));
    // Only for nix
    env.push(("HOME".to_string(), "/".to_string()));
    env.push(("NIX_REMOTE".to_string(), getenv("NIX_REMOTE").unwrap()));
    env.push(("NIX_PATH".to_string(), getenv("NIX_PATH").unwrap()));
    // End of nix hacks
    env.push(("container_name".to_string(), task.name.clone()));
    env.push(("container_dir".to_string(),
        format!("{}", container_dir.display())));
    env.push(("container_root".to_string(),
        format!("{}", container_tmp.display())));
    for (k, v) in container.settings.iter() {
        env.push((builder + "_" + *k, v.clone()));
    }
    match Command::new(bexe).env(env.as_slice())
        .stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2))
        .status() {
        Ok(ExitStatus(0)) => {}
        Ok(x) => return Err(format!("Builder exited with status {}", x)),
        Err(x) => return Err(format!("Can't spawn: {}", x)),
    }

    let container_old = container_dir.join(".oldroot");
    if container_root.exists() {
        if container_old.exists() {
            match rmdir_recursive(&container_old) {
                Ok(()) => {}
                Err(x) => return Err(format!("Can't remove old root: {}", x)),
            }
        }
        match rename(&container_root, &container_old) {
            Ok(()) => {}
            Err(x) => return Err(format!("Can't rename old root: {}", x)),
        }
    }

    match rename(&container_tmp, &container_root) {
        Ok(()) => {}
        Err(x) => return Err(format!("Can't rename root: {}", x)),
    }

    if container_old.exists() {
        match rmdir_recursive(&container_old) {
            Ok(()) => {}
            Err(x) => return Err(format!("Can't remove old root: {}", x)),
        }
    }

    task.stderr.write_line("Done").ok();

    return Ok(());
}
