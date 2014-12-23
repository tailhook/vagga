use std::rc::Rc;
use std::os::{getenv};
use std::io::ALL_PERMISSIONS;
use std::io::fs::{mkdir};
use std::io::fs::PathExtensions;
use std::io::stdio::{stdout, stderr};
use std::collections::TreeMap;

use argparse::{ArgumentParser, List};

use config::{Config, Settings, Container};
use config::command::CommandInfo;
use container::root::change_root;
use container::mount::{bind_mount, unmount, mount_system_dirs};
use container::uidmap::{map_users, Ranges, Singleton};
use container::monitor::{Monitor, Executor};
use container::monitor::{Killed, Exit};
use container::container::{Command};

use super::build;
use super::setup;

struct RunCommand<'a> {
    env: &'a TreeMap<String, String>,
    cmd: Path,
    args: Vec<String>,
    settings: &'a Settings,
    container: &'a Container,
    command: &'a CommandInfo,
}

impl<'a> Executor for RunCommand<'a> {
    fn command(&self) -> Command {
        let mut cmd = Command::new("run".to_string(), &self.cmd);
        cmd.args(self.args.as_slice());
        cmd.set_uidmap(self.settings.uid_map.as_ref()
            .map(|&(ref x, ref y)| Ranges(x.clone(), y.clone()))
            .unwrap_or(Singleton(0, 0)));
        if let Some(ref wd) = self.command.work_dir {
            cmd.set_workdir(&Path::new("/work").join(wd.as_slice()));
        } else {
            // TODO(tailhook) set workdir to current one
        }
        cmd.set_env("TERM".to_string(),
                    getenv("TERM").unwrap_or("dumb".to_string()));
        for (ref k, ref v) in self.env.iter() {
            cmd.set_env(k.to_string(), v.to_string());
        }
        return cmd;
    }
}

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
    try!(mount_system_dirs()
        .map_err(|e| format!("Error mounting system dirs: {}", e)));
    try!(change_root(&tgtroot, &tgtroot.join("tmp"))
         .map_err(|e| format!("Error changing root: {}", e)));
    try!(unmount(&Path::new("/work/.vagga/.mnt"))
         .map_err(|e| format!("Error unmounting `.vagga/.mnt`: {}", e)));
    try!(unmount(&Path::new("/tmp"))
         .map_err(|e| format!("Error unmounting old root: {}", e)));

    let env = try!(setup::get_environment(cconfig));
    let mut mon = Monitor::new();
    let mut cmd = Path::new(cmdline.remove(0).unwrap().as_slice());
    if cmd.is_absolute() {
    } else {
        let paths = [
            "/bin",
            "/usr/bin",
            "/usr/local/bin",
            "/sbin",
            "/usr/sbin",
            "/usr/local/sbin",
        ];
        for path in paths.iter() {
            let path = Path::new(*path).join(&cmd);
            if path.exists() {
                cmd = path;
                break;
            }
        }
        if !cmd.is_absolute() {
            return Err(format!("Command {} not found in {}",
                cmd.display(), paths.as_slice()));
        }
    }

    mon.add(Rc::new("run".to_string()), box RunCommand {
        env: &env,
        cmd: cmd,
        args: cmdline,
        settings: &settings,
        container: cconfig,
        command: command,
    });
    match mon.run() {
        Killed => return Ok(1),
        Exit(val) => return Ok(val),
    };
}
