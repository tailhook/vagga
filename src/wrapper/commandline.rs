use std::env;
use std::fs::read_link;
use std::io::{stdout, stderr};
use std::path::Path;
use std::str::FromStr;

use libc::pid_t;
use argparse::{ArgumentParser};
use unshare::{Command, UidMap};

use config::command::CommandInfo;
use config::command::WriteMode;
use container::uidmap::{map_users};

use super::setup;
use super::Wrapper;
use super::util::find_cmd;
use process_util::{convert_status, set_uidmap};


pub fn commandline_cmd(command: &CommandInfo,
    wrapper: &Wrapper, mut cmdline: Vec<String>)
    -> Result<i32, String>
{
    // TODO(tailhook) detect other shells too
    let has_args = command.accepts_arguments
            .unwrap_or(&command.run[0][..] != "/bin/sh");
    let mut args = Vec::new();
    if !has_args {
        let mut ap = ArgumentParser::new();
        ap.set_description(command.description.as_ref()
            .map(|x| &x[..]).unwrap_or(""));
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
    let mut cmdline = command.run.clone();
    cmdline.extend(args.into_iter());

    let pid: pid_t = try!(read_link(&Path::new("/proc/self"))
        .map_err(|e| format!("Can't read /proc/self: {}", e))
        .and_then(|v| v.to_str().and_then(|x| FromStr::from_str(x).ok())
            .ok_or(format!("Can't parse pid: {:?}", v))));
    try!(setup::setup_base_filesystem(
        wrapper.project_root, wrapper.ext_settings));

    let cconfig = try!(wrapper.config.containers.get(&command.container)
        .ok_or(format!("Container {} not found", command.container)));
    let uid_map = try!(map_users(wrapper.settings,
        &cconfig.uids, &cconfig.gids));

    let write_mode = match command.write_mode {
        WriteMode::read_only => setup::WriteMode::ReadOnly,
        WriteMode::transient_hard_link_copy
        => setup::WriteMode::TransientHardlinkCopy(pid),
    };
    let cont_ver = wrapper.root.as_ref().unwrap();
    try!(setup::setup_filesystem(cconfig, write_mode, &cont_ver));

    let mut env = try!(setup::get_environment(cconfig, &wrapper.settings));
    for (k, v) in command.environ.iter() {
        env.insert(k.clone(), v.clone());
    }
    let cpath = try!(find_cmd(&cmdline.remove(0), &env));

    let mut cmd = Command::new(&cpath);
    cmd.args(&cmdline);
    if let Some(euid) = command.external_user_id {
        cmd.set_id_maps(vec![
            UidMap {
            inside_uid: command.user_id,
            outside_uid: euid,
            count: 1 }
            ], vec![]);
        cmd.uid(command.user_id);
    } else {
        set_uidmap(&mut cmd, &uid_map, false);
        cmd.uid(command.user_id);
    }
    if let Some(ref wd) = command.work_dir {
        cmd.current_dir(Path::new("/work").join(&wd));
    } else {
        cmd.current_dir(env::var("PWD").unwrap_or("/work".to_string()));
    }
    for (ref k, ref v) in env.iter() {
        cmd.env(k, v);
    }

    match cmd.status() {
        Ok(s) => Ok(convert_status(s)),
        Err(e) => Err(format!("Error running {:?}: {}", cmd, e)),
    }
}
