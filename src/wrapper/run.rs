use std::env;
use std::fs::{read_link};
use std::io::{stdout, stderr};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use libc::pid_t;
use argparse::{ArgumentParser, Store, List, StoreTrue};
use unshare::{Command};

use container::uidmap::{map_users};
use super::setup;
use super::Wrapper;
use path_util::PathExt;
use process_util::{set_uidmap, copy_env_vars, run_and_wait};


pub fn run_command_cmd(wrapper: &Wrapper,
    cmdline: Vec<String>, user_ns: bool, tty_fd: Option<i32>)
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
    let pid: pid_t = try!(read_link(&Path::new("/proc/self"))
        .map_err(|e| format!("Can't read /proc/self: {}", e))
        .and_then(|v| v.to_str().and_then(|x| FromStr::from_str(x).ok())
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
    let container_ver = wrapper.root.as_ref().unwrap();
    try!(setup::setup_filesystem(cconfig, write_mode, container_ver));

    let env = try!(setup::get_environment(cconfig, &wrapper.settings));
    let mut cpath = PathBuf::from(&command);
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
                cpath.display(), &paths));
        }
    }

    let mut cmd = Command::new(cpath);
    cmd.args(&args);
    cmd.current_dir(&env::var("_VAGGA_WORKDIR")
                    .unwrap_or("/work".to_string()));
    uid_map.map(|x| set_uidmap(&mut cmd, &x, false));
    cmd.env_clear();
    copy_env_vars(&mut cmd, &wrapper.settings);
    for (ref k, ref v) in env.iter() {
        cmd.env(k.to_string(), v.to_string());
    }

    if let Some(_) = tty_fd {
        cmd.make_group_leader(true);
    }

    run_and_wait(&mut cmd, tty_fd)
}
