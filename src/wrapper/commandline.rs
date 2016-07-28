use std::env;
use std::fs::read_link;
use std::io::{stdout, stderr};
use std::path::Path;
use std::str::FromStr;

use libc::pid_t;
use argparse::{ArgumentParser};
use unshare::{Command};

use config::command::CommandInfo;
use config::command::WriteMode;

use super::setup;
use super::Wrapper;
use super::util::{find_cmd, warn_if_data_container};
use process_util::{run_and_wait, convert_status, copy_env_vars};
use process_util::{set_fake_uidmap};


pub fn commandline_cmd(command: &CommandInfo,
    wrapper: &Wrapper, mut cmdline: Vec<String>)
    -> Result<i32, String>
{
    if command.run.len() == 0 {
        return Err(format!(
            r#"Command has empty "run" parameter. Nothing to run."#));
    }
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

    let write_mode = match command.write_mode {
        WriteMode::read_only => setup::WriteMode::ReadOnly,
        WriteMode::transient_hard_link_copy
        => setup::WriteMode::TransientHardlinkCopy(pid),
    };
    let cont_ver = wrapper.root.as_ref().unwrap();
    let mut setup_info = setup::SetupInfo::from_container(&cconfig);
    setup_info
        .volumes(&command.volumes)
        .write_mode(write_mode);
    warn_if_data_container(&cconfig);
    try!(setup::setup_filesystem(&setup_info, &cont_ver));

    let mut env = try!(setup::get_environment(cconfig, &wrapper.settings));
    for (k, v) in command.environ.iter() {
        env.insert(k.clone(), v.clone());
    }
    let cpath = try!(find_cmd(&cmdline.remove(0), &env));

    let mut cmd = Command::new(&cpath);
    cmd.args(&cmdline);
    cmd.env_clear();
    copy_env_vars(&mut cmd, &wrapper.settings);
    if let Some(euid) = command.external_user_id {
        try!(set_fake_uidmap(&mut cmd, command.user_id, euid));
    }
    cmd.uid(command.user_id);
    cmd.gid(command.group_id);
    cmd.groups(command.supplementary_gids.clone());
    if let Some(ref wd) = command.work_dir {
        cmd.current_dir(Path::new("/work").join(&wd));
    } else {
        cmd.current_dir(env::var("_VAGGA_WORKDIR")
                        .unwrap_or("/work".to_string()));
    }
    for (ref k, ref v) in env.iter() {
        cmd.env(k, v);
    }

    run_and_wait(&mut cmd)
    .map(convert_status)
}
