use std::env;
use std::fs::{read_link, create_dir, File};
use std::io::{stdout, stderr};
use std::path::Path;
use std::str::FromStr;

use libc::pid_t;
use argparse::{ArgumentParser, Store};

use super::super::config::command::{SuperviseInfo, CommandInfo, WriteMode};
use super::super::config::command::ChildCommand as CC;
use super::super::container::uidmap::{map_users};
use super::super::container::uidmap::Uidmap::Ranges;
use super::super::container::monitor::{Monitor};
use super::super::container::monitor::MonitorResult::{Killed, Exit};
use super::super::container::container::{Command};
use super::super::container::vagga::container_ver;
use super::Wrapper;
use super::util::find_cmd;
use super::setup;


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
    let pid: pid_t = try!(readlink(&Path::new("/proc/self"))
        .map_err(|e| format!("Can't read /proc/self: {}", e))
        .and_then(|v| v.as_str().and_then(|x| FromStr::from_str(x).ok())
            .ok_or(format!("Can't parse pid: {:?}", v))));
    try!(setup::setup_base_filesystem(
        wrapper.project_root, wrapper.ext_settings));

    let childtype = try!(command.children.get(&child)
        .ok_or(format!("Child {} not found", child)));
    match childtype {
        &CC::Command(ref info) => supervise_child_command(cname,
            &child, false, info, wrapper, command, pid),
        &CC::BridgeCommand(ref info) => supervise_child_command(cname,
            &child, true, info, wrapper, command, pid),
    }
}

fn _write_hosts(supervise: &SuperviseInfo) -> Result<(), String> {
    let basedir = Path::new("/tmp/vagga");
    if !basedir.is_dir() {
        try!(mkdir(&basedir, ALL_PERMISSIONS)
            .map_err(|e| format!("Can't create dir: {}", e)));
    }
    let mut file = File::create(&basedir.join("hosts"));
    try!((writeln!(&mut file, "127.0.0.1 localhost"))
         .map_err(|e| format!("Error writing hosts: {}", e)));
    for (subname, subcommand) in supervise.children.iter() {
        if let &CC::Command(ref cmd) = subcommand {
            if let Some(ref netw) = cmd.network {
                // TODO(tailhook) support multiple commands with same IP
                if let Some(ref val) = netw.hostname {
                    try!((writeln!(&mut file, "{} {} {}", netw.ip,
                                    val, subname))
                         .map_err(|e| format!("Error writing hosts: {}", e)));
                } else {
                    try!((writeln!(&mut file, "{} {}", netw.ip, subname))
                         .map_err(|e| format!("Error writing hosts: {}", e)));
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
    let cconfig = try!(wrapper.config.containers.get(&command.container)
        .ok_or(format!("Container {} not found", command.container)));
    let uid_map = try!(map_users(wrapper.settings,
        &cconfig.uids, &cconfig.gids));


    let write_mode = match command.write_mode {
        WriteMode::read_only => setup::WriteMode::ReadOnly,
        WriteMode::transient_hard_link_copy
        => setup::WriteMode::TransientHardlinkCopy(pid),
    };
    let cont_ver = try!(container_ver(&command.container));
    try!(setup::setup_filesystem(cconfig, write_mode, cont_ver.as_slice()));

    try!(_write_hosts(supervise));

    let mut env = try!(setup::get_environment(cconfig));
    for (k, v) in command.environ.iter() {
        env.insert(k.clone(), v.clone());
    }
    let mut cmdline = command.run.clone();
    let cpath = try!(find_cmd(cmdline.remove(0).as_slice(), &env));

    let mut cmd = Command::new(name.to_string(), &cpath);
    cmd.args(cmdline.as_slice());
    cmd.set_env("VAGGA_COMMAND".to_string(), cmdname.to_string());
    cmd.set_env("VAGGA_SUBCOMMAND".to_string(), name.to_string());
    if !bridge {
        if let Some(euid) = command.external_user_id {
            cmd.set_uidmap(Ranges(vec!(
                (command.user_id as u32, euid as u32, 1)), vec!((0, 0, 1))));
            cmd.set_user_id(command.user_id);
        } else {
            cmd.set_user_id(command.user_id);
            cmd.set_uidmap(uid_map.clone());
        }
    }
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
