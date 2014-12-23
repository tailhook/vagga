use std::rc::Rc;
use std::os::{getenv};
use std::io::ALL_PERMISSIONS;
use std::io::fs::{mkdir};
use std::io::fs::PathExtensions;
use std::io::stdio::{stdout, stderr};

use argparse::{ArgumentParser, Store, List};

use container::root::change_root;
use container::mount::{bind_mount, unmount, mount_system_dirs};
use container::uidmap::{map_users, Ranges, Singleton};
use container::monitor::{Monitor, Executor};
use container::monitor::{Killed, Exit};
use container::container::{Command};
use config::{Settings, Config};

use super::build;

pub static DEFAULT_PATH: &'static str =
    "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";


struct RunCommand<'a> {
    cmd: Path,
    args: Vec<String>,
    settings: &'a Settings,
}

impl<'a> Executor for RunCommand<'a> {
    fn command(&self) -> Command {
        let mut cmd = Command::new("run".to_string(), &self.cmd);
        cmd.args(self.args.as_slice());
        cmd.set_uidmap(self.settings.uid_map.as_ref()
            .map(|&(ref x, ref y)| Ranges(x.clone(), y.clone()))
            .unwrap_or(Singleton(0, 0)));
        cmd.set_env("TERM".to_string(),
                    getenv("TERM").unwrap_or("dumb".to_string()));
        if let Some(x) = getenv("RUST_LOG") {
            cmd.set_env("RUST_LOG".to_string(), x);
        }
        if let Some(x) = getenv("RUST_BACKTRACE") {
            cmd.set_env("RUST_BACKTRACE".to_string(), x);
        }
        return cmd;
    }
}

pub fn run_command(settings: &Settings, container: &String,
    command: &String, args: &[String])
    -> Result<int, String>
{
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

    let mut mon = Monitor::new();
    let mut cmd = Path::new(command.as_slice());
    let args = args.clone().to_vec();
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
        cmd: cmd,
        args: args,
        settings: settings,
    });
    match mon.run() {
        Killed => return Ok(1),
        Exit(val) => return Ok(val),
    };
}

pub fn run_command_cmd(config: &Config, settings: &Settings,
    cmdline: Vec<String>)
    -> Result<int, String>
{
    let mut container: String = "".to_string();
    let mut command: String = "".to_string();
    let mut args = Vec::new();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Runs arbitrary command inside the container
            ");
        ap.refer(&mut container)
            .add_argument("container_name", box Store::<String>,
                "Container name to build");
        ap.refer(&mut command)
            .add_argument("command", box Store::<String>,
                "Command to run inside the container");
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
    }
    let cconfig = try!(config.containers.find(&container)
        .ok_or(format!("Container {} not found", container)));
    let settings = try!(map_users(settings, &cconfig.uids, &cconfig.gids));
    return build::build_container(&container, false, &settings)
        .and_then(|cont|
            run_command(&settings, &cont, &command, args.as_slice()));
}
