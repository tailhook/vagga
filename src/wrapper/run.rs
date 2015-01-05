use std::os::{getenv};
use std::io::fs::PathExtensions;
use std::io::stdio::{stdout, stderr};

use argparse::{ArgumentParser, Store, List};

use config::{Container};
use container::uidmap::{map_users, Uidmap};
use container::monitor::{Monitor};
use container::monitor::{Killed, Exit};
use container::container::{Command};

use super::build;
use super::setup;
use super::Wrapper;

pub static DEFAULT_PATH: &'static str =
    "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";


pub fn run_command(uid_map: &Option<Uidmap>, cname: &String,
    container: &Container, command: &String, args: &[String])
    -> Result<int, String>
{
    try!(setup::setup_filesystem(container, cname.as_slice()));

    let env = try!(setup::get_environment(container));
    let mut cpath = Path::new(command.as_slice());
    let args = args.clone().to_vec();
    if cpath.is_absolute() {
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
            return Err(format!("Command {} not found in {}",
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

pub fn run_command_cmd(wrapper: &Wrapper, cmdline: Vec<String>, user_ns: bool)
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
        /* TODO(tailhook) implement environment settings
        ap.refer(&mut env.set_env)
          .add_option(&["-E", "--env", "--environ"], box Collect::<String>,
                "Set environment variable for running command")
          .metavar("NAME=VALUE");
        ap.refer(&mut env.propagate_env)
          .add_option(&["-e", "--use-env"], box Collect::<String>,
                "Propagate variable VAR into command environment")
          .metavar("VAR");
        */
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
    try!(setup::setup_base_filesystem(
        wrapper.project_root, wrapper.ext_settings));
    let cconfig = try!(wrapper.config.containers.find(&container)
        .ok_or(format!("Container {} not found", container)));
    let uid_map = if user_ns {
            Some(try!(map_users(wrapper.settings,
                &cconfig.uids, &cconfig.gids)))
        } else {
            None
        };
    return build::build_container(&container, false, wrapper)
        .and_then(|cont| run_command(&uid_map, &cont, cconfig,
                                     &command, args.as_slice()));
}
