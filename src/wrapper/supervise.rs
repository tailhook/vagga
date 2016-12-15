use std::env;
use std::fs::{read_link, File};
use std::io::{stdout, stderr, Write};
use std::path::Path;
use std::str::FromStr;

use libc::pid_t;
use argparse::{ArgumentParser, Store};
use unshare::Command;

use super::super::config::command::{SuperviseInfo, CommandInfo, WriteMode};
use super::super::config::command::ChildCommand as CC;
use super::Wrapper;
use super::util::{find_cmd, warn_if_data_container};
use super::setup;
use file_util::Dir;
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

fn _write_hosts(supervise: &SuperviseInfo) -> Result<(), String> {
    let basedir = Path::new("/tmp/vagga");
    try_msg!(Dir::new(&basedir).create(),
             "Can't create dir: {err}");
    let mut file = try_msg!(File::create(&basedir.join("hosts")),
        "Can't create hosts file: {err}");
    (writeln!(&mut file, "127.0.0.1 localhost"))
         .map_err(|e| format!("Error writing hosts: {}", e))?;
    for (subname, subcommand) in supervise.children.iter() {
        if let &CC::Command(ref cmd) = subcommand {
            if let Some(ref netw) = cmd.network {
                // TODO(tailhook) support multiple commands with same IP
                if let Some(ref val) = netw.hostname {
                    writeln!(&mut file, "{} {} {}", netw.ip, val, subname)
                         .map_err(|e| format!("Error writing hosts: {}", e))?;
                } else {
                    writeln!(&mut file, "{} {}", netw.ip, subname)
                         .map_err(|e| format!("Error writing hosts: {}", e))?;
                }
            }
        }
    }
    return Ok(());
}

fn supervise_child_command(cmdname: &String, name: &String, bridge: bool,
    command: &CommandInfo, wrapper: &Wrapper, supervise: &SuperviseInfo,
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

    _write_hosts(supervise)?;

    let env = setup::get_environment(&wrapper.settings, cconfig,
        Some(&command))?;
    let mut cmdline = command.run.clone();
    let cpath = find_cmd(&cmdline.remove(0), &env)?;

    let mut cmd = Command::new(&cpath);
    cmd.args(&cmdline);
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
