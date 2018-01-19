use std::env;
use std::fs::{read_link};
use std::io::{stdout, stderr};
use std::path::Path;
use std::str::FromStr;

use libc::pid_t;
use argparse::{ArgumentParser, Store};

use super::super::config::command::{SuperviseInfo, CommandInfo, WriteMode};
use super::super::config::command::ChildCommand as CC;
use super::Wrapper;
use super::util::{gen_command, warn_if_data_container};
use super::setup;
use process_util::{run_and_wait, convert_status, copy_env_vars};
use process_util::{set_fake_uidmap, cmd_err};


pub fn supervise_cmd(cname: &String, command: &SuperviseInfo,
    wrapper: &Wrapper, cmdline: Vec<String>)
    -> Result<i32, String>
{
    let mut child = "".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Runs a single command from supervision suite");
        ap.refer(&mut child)
            .add_argument("child", Store,
                "Child to run")
            .required();
        match ap.parse(cmdline, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => {
                return Ok(122);
            }
        }
    }
    let pid: pid_t = read_link(&Path::new("/proc/self"))
        .map_err(|e| format!("Can't read /proc/self: {}", e))
        .and_then(|v| v.to_str().and_then(|x| FromStr::from_str(x).ok())
            .ok_or(format!("Can't parse pid: {:?}", v)))?;
    setup::setup_base_filesystem(
        wrapper.project_root, wrapper.ext_settings)?;

    let childtype = command.children.get(&child)
        .ok_or(format!("Child {} not found", child))?;
    match childtype {
        &CC::Command(ref info) => supervise_child_command(cname,
            &child, false, info, wrapper, command, pid),
        &CC::BridgeCommand(ref info) => supervise_child_command(cname,
            &child, true, info, wrapper, command, pid),
    }
}

fn supervise_child_command(cmdname: &String, name: &String, bridge: bool,
    command: &CommandInfo, wrapper: &Wrapper, _supervise: &SuperviseInfo,
    pid: pid_t)
    -> Result<i32, String>
{
    let cconfig = wrapper.config.containers.get(&command.container)
        .ok_or(format!("Container {} not found", command.container))?;

    let write_mode = match command.write_mode {
        WriteMode::read_only => setup::WriteMode::ReadOnly,
        WriteMode::transient_hard_link_copy
        => setup::WriteMode::TransientHardlinkCopy(pid),
    };
    let cont_ver = wrapper.root.as_ref().unwrap();
    let mut setup_info = setup::SetupInfo::from_container(&cconfig);
    setup_info.volumes(&command.volumes)
        .write_mode(write_mode);
    warn_if_data_container(&cconfig);
    setup::setup_filesystem(&setup_info, &cont_ver)?;

    let env = setup::get_environment(&wrapper.settings, cconfig,
        Some(&command))?;
    let mut cmd = gen_command(&cconfig.default_shell, &command.run, &env)?;
    cmd.env_clear();
    copy_env_vars(&mut cmd, &wrapper.settings);
    cmd.env("VAGGA_COMMAND", cmdname);
    cmd.env("VAGGA_SUBCOMMAND", name);
    if !bridge {
        if let Some(euid) = command.external_user_id {
            set_fake_uidmap(&mut cmd, command.user_id, euid)?;
        }
        cmd.uid(command.user_id);
    }
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
    .map_err(|e| cmd_err(&cmd, e))
}
