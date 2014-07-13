use std::io;
use std::os::getenv;
use std::io::fs::{mkdir, rmdir_recursive, rename};
use std::io::process::{ExitStatus, Command, InheritFd};
use std::io::stdio::{stdout, stderr};

use argparse::{ArgumentParser, Store};

use super::env::Environ;
use super::options::env_options;


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

pub fn build_command(environ: &mut Environ, args: Vec<String>)
    -> Result<int, String>
{
    let mut cname = "devel".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut cname)
            .add_argument("container", box Store::<String>,
                "A name of the container to build")
            .required();
        env_options(environ, &mut ap);
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => return Ok(122),
        }
    }
    let container = try!(environ.get_container(&cname));

    info!("Building {}", cname);

    let builder = &container.builder;
    let bexe = environ.vagga_dir.join_many(
        ["build-scripts", builder.as_slice()]);
    if !bexe.exists() {
        return Err(format!("Builder {} does not exist", builder));
    }

    info!("Builder {}", bexe.display());

    let mut env = Vec::new();
    let container_dir = environ.project_root
        .join_many([".vagga", cname.as_slice()]);
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
    env.push(("container_name".to_string(), cname.clone()));
    env.push(("project_root".to_string(),
        format!("{}", environ.project_root.display())));
    env.push(("container_dir".to_string(),
        format!("{}", container_dir.display())));
    env.push(("container_root".to_string(),
        format!("{}", container_tmp.display())));
    for (k, v) in container.settings.iter() {
        env.push((builder + "_" + *k, v.clone()));
    }
    match Command::new(bexe).env(env.as_slice())
        .stdin(InheritFd(0)).stdout(InheritFd(1)).stderr(InheritFd(2))
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

    info!("Done building {}", cname);

    return Ok(0);
}
