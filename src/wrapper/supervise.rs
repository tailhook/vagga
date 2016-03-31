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
use super::super::container::uidmap::{map_users};
use super::super::container::uidmap::Uidmap::Ranges;
use super::Wrapper;
use super::util::find_cmd;
use super::setup;
use super::super::file_util::create_dir;
use process_util::{set_uidmap, run_and_wait, convert_status, copy_env_vars};


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
    let pid: pid_t = try!(read_link(&Path::new("/proc/self"))
        .map_err(|e| format!("Can't read /proc/self: {}", e))
        .and_then(|v| v.to_str().and_then(|x| FromStr::from_str(x).ok())
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
    try_msg!(create_dir(&basedir, false),
             "Can't create dir: {err}");
    let mut file = try_msg!(File::create(&basedir.join("hosts")),
        "Can't create hosts file: {err}");
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
    let cont_ver = wrapper.root.as_ref().unwrap();
    try!(setup::setup_filesystem(cconfig, write_mode, &cont_ver));

    try!(_write_hosts(supervise));

    let mut env = try!(setup::get_environment(cconfig, &wrapper.settings));
    for (k, v) in command.environ.iter() {
        env.insert(k.clone(), v.clone());
    }
    let mut cmdline = command.run.clone();
    let cpath = try!(find_cmd(&cmdline.remove(0), &env));

    let mut cmd = Command::new(&cpath);
    cmd.args(&cmdline);
    cmd.env_clear();
    copy_env_vars(&mut cmd, &wrapper.settings);
    cmd.env("VAGGA_COMMAND", cmdname);
    cmd.env("VAGGA_SUBCOMMAND", name);
    if !bridge {
        let curmap = if let Some(euid) = command.external_user_id {
            Ranges(vec!(
                (command.user_id as u32, euid as u32, 1)), vec!((0, 0, 1)))
        } else {
            uid_map
        };
        set_uidmap(&mut cmd, &curmap, false);
        cmd.uid(command.user_id);
    }
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
    .map_err(|e| format!("Error running {:?}: {}", cmd, e))
}
