use std::os::getenv;
use std::io::stdio::{stdout, stderr};

use argparse::{ArgumentParser};

use config::command::CommandInfo;
use container::uidmap::{map_users};
use container::monitor::{Monitor};
use container::monitor::{Killed, Exit};
use container::container::{Command};

use super::build;
use super::setup;
use super::Wrapper;
use super::util::find_cmd;


pub fn commandline_cmd(command: &CommandInfo,
    wrapper: &Wrapper, mut cmdline: Vec<String>)
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

    try!(setup::setup_base_filesystem(
        wrapper.project_root, wrapper.ext_settings));

    let cconfig = try!(wrapper.config.containers.find(&command.container)
        .ok_or(format!("Container {} not found", command.container)));
    let uid_map = try!(map_users(wrapper.settings,
        &cconfig.uids, &cconfig.gids));
    let container = try!(build::build_container(&command.container,
                                                false, wrapper));

    try!(setup::setup_filesystem(cconfig, container.as_slice()));

    let mut env = try!(setup::get_environment(cconfig));
    for (k, v) in command.environ.iter() {
        env.insert(k.clone(), v.clone());
    }
    let cpath = try!(find_cmd(cmdline.remove(0).unwrap().as_slice(), &env));

    let mut cmd = Command::new("run".to_string(), &cpath);
    cmd.args(cmdline.as_slice());
    cmd.set_uidmap(uid_map.clone());
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
