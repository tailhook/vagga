use std::os::{getenv};
use std::str::FromStr;
use std::old_path::BytesContainer;
use std::old_io::fs::PathExtensions;
use std::old_io::fs::readlink;
use std::old_io::stdio::{stdout, stderr};
use libc::pid_t;

use argparse::{ArgumentParser, Store, List, StoreTrue};

use config::{Container};
use container::uidmap::{map_users, Uidmap};
use container::monitor::{Monitor};
use container::monitor::MonitorResult::{Killed, Exit};
use container::container::{Command};
use container::vagga::container_ver;

use super::build;
use super::setup;
use super::Wrapper;

pub static DEFAULT_PATH: &'static str =
    "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";


pub fn run_command_cmd(wrapper: &Wrapper, cmdline: Vec<String>, user_ns: bool)
    -> Result<i32, String>
{
    let mut container: String = "".to_string();
    let mut command: String = "".to_string();
    let mut args = Vec::<String>::new();
    let mut copy = false;
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Runs arbitrary command inside the container
            ");
        ap.refer(&mut copy)
            .add_option(&["-W", "--writeable"], StoreTrue,
                "Create translient writeable container for running the command.
                 Currently we use hard-linked copy of the container, so it's
                 dangerous for some operations. Still it's ok for installing
                 packages or similar tasks");
        ap.refer(&mut container)
            .add_argument("container_name", Store,
                "Container name to build");
        ap.refer(&mut command)
            .add_argument("command", Store,
                "Command to run inside the container");
        ap.refer(&mut args)
            .add_argument("args", List,
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
    let pid: pid_t = try!(readlink(&Path::new("/proc/self"))
        .map_err(|e| format!("Can't read /proc/self: {}", e))
        .and_then(|v| v.as_str().and_then(|x| FromStr::from_str(x).ok())
            .ok_or(format!("Can't parse pid: {:?}", v))));
    try!(setup::setup_base_filesystem(
        wrapper.project_root, wrapper.ext_settings));
    let cconfig = try!(wrapper.config.containers.get(&container)
        .ok_or(format!("Container {} not found", container)));
    let uid_map = if user_ns {
        Some(try!(map_users(wrapper.settings,
            &cconfig.uids, &cconfig.gids)))
    } else {
        None
    };

    let write_mode = match copy {
        false => setup::WriteMode::ReadOnly,
        true => setup::WriteMode::TransientHardlinkCopy(pid),
    };
    let cont_ver = try!(container_ver(&container));
    try!(setup::setup_filesystem(cconfig, write_mode, cont_ver.as_slice()));

    let env = try!(setup::get_environment(cconfig));
    let mut cpath = Path::new(command.as_slice());
    let args = args.clone().to_vec();
    if command.contains("/") {
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
            let path = Path::new(*path).join(&cpath);
            if path.exists() {
                cpath = path;
                break;
            }
        }
        if !cpath.is_absolute() {
            return Err(format!("Command {} not found in {:?}",
                cpath.display(), paths.as_slice()));
        }
    }

    let mut cmd = Command::new("run".to_string(), &cpath);
    cmd.args(args.as_slice());
    cmd.set_workdir(&Path::new(getenv("PWD").unwrap_or("/work".to_string())));
    uid_map.as_ref().map(|v| cmd.set_uidmap(v.clone()));
    cmd.set_env("TERM".to_string(),
                getenv("TERM").unwrap_or("dumb".to_string()));
    for (ref k, ref v) in env.iter() {
        cmd.set_env(k.to_string(), v.to_string());
    }

    match Monitor::run_command(cmd) {
        Killed => return Ok(1),
        Exit(val) => return Ok(val),
    };
}
