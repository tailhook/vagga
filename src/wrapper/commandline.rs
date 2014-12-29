use std::os::getenv;
use std::io::ALL_PERMISSIONS;
use std::io::fs::{mkdir};
use std::io::fs::PathExtensions;
use std::io::stdio::{stdout, stderr};

use argparse::{ArgumentParser, List};

use config::{Config, Settings};
use config::command::CommandInfo;
use container::root::change_root;
use container::mount::{bind_mount, unmount, mount_system_dirs, remount_ro};
use container::uidmap::{map_users, Ranges, Singleton};
use container::monitor::{Monitor};
use container::monitor::{Killed, Exit};
use container::container::{Command};

use super::build;
use super::setup;


pub fn commandline_cmd(command: &CommandInfo, config: &Config,
    settings: &Settings, mut cmdline: Vec<String>)
    -> Result<int, String>
{
    // TODO(tailhook) detect other shells too
    let has_args = command.accepts_arguments
            .unwrap_or(command.run[0].as_slice() != "/bin/sh");
    let mut args = Vec::new();
    if !has_args {
        let mut ap = ArgumentParser::new();
        ap.set_description(command.description.as_ref()
            .map(|x| x.as_slice()).unwrap_or(""));
        ap.refer(&mut args)
            .add_argument("args", box List::<String>,
                "Arguments for the command");
        ap.stop_on_first_argument(true);
        match ap.parse(cmdline, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => {
                return Ok(122);
            }
        }
    } else {
        cmdline.remove(0);
        args.extend(cmdline.into_iter());
    }
    let mut cmdline = command.run + args;
    let cconfig = try!(config.containers.find(&command.container)
        .ok_or(format!("Container {} not found", command.container)));
    let settings = try!(map_users(settings, &cconfig.uids, &cconfig.gids));
    let container = try!(build::build_container(&command.container,
                                                false, &settings));

    let tgtroot = Path::new("/vagga/root");
    if !tgtroot.exists() {
        try!(mkdir(&tgtroot, ALL_PERMISSIONS)
             .map_err(|x| format!("Error creating directory: {}", x)));
    }
    try!(bind_mount(&Path::new("/vagga/roots")
                     .join(container.as_slice()).join("root"),
                    &tgtroot)
         .map_err(|e| format!("Error bind mount: {}", e)));
    try!(remount_ro(&tgtroot));
    try!(mount_system_dirs()
        .map_err(|e| format!("Error mounting system dirs: {}", e)));
    try!(change_root(&tgtroot, &tgtroot.join("tmp"))
         .map_err(|e| format!("Error changing root: {}", e)));
    try!(unmount(&Path::new("/work/.vagga/.mnt"))
         .map_err(|e| format!("Error unmounting `.vagga/.mnt`: {}", e)));
    try!(unmount(&Path::new("/tmp"))
         .map_err(|e| format!("Error unmounting old root: {}", e)));

    let mut env = try!(setup::get_environment(cconfig));
    for (k, v) in command.environ.iter() {
        env.insert(k.clone(), v.clone());
    }
    let mut cpath = Path::new(cmdline.remove(0).unwrap().as_slice());
    if cpath.is_absolute() {
    } else {
        if let Some(paths) = env.find(&"PATH".to_string()) {
            for dir in paths.as_slice().split(':') {
                let path = Path::new(dir);
                if !path.is_absolute() {
                    warn!("All items in PATH must be absolute, not {}",
                          path.display());
                    continue;
                }
                let path = path.join(&cpath);
                if path.exists() {
                    cpath = path;
                }
            }
            if !cpath.is_absolute() {
                return Err(format!("Command {} not found in {}",
                    cpath.display(), paths.as_slice()));
            }
        } else {
            return Err(format!("Command {} is not absolute and no PATH set",
                cpath.display()));
        }
    }

    let mut cmd = Command::new("run".to_string(), &cpath);
    cmd.args(cmdline.as_slice());
    cmd.set_uidmap(settings.uid_map.as_ref()
        .map(|&(ref x, ref y)| Ranges(x.clone(), y.clone()))
        .unwrap_or(Singleton(0, 0)));
    if let Some(ref wd) = command.work_dir {
        cmd.set_workdir(&Path::new("/work").join(wd.as_slice()));
    } else {
        cmd.set_workdir(&Path::new(
            getenv("PWD").unwrap_or("/work".to_string())));
    }
    for (ref k, ref v) in env.iter() {
        cmd.set_env(k.to_string(), v.to_string());
    }

    match Monitor::run_command(cmd) {
        Killed => return Ok(1),
        Exit(val) => return Ok(val),
    };
}
