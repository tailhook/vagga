use std::io::ALL_PERMISSIONS;
use std::os::getenv;
use std::io::fs::{readlink, mkdir, File, PathExtensions};
use std::io::stdio::{stdout, stderr};

use argparse::{ArgumentParser, Store};

use config::command::{SuperviseInfo, CommandInfo};
use config::command::child::{Command, BridgeCommand};
use container::uidmap::{map_users};
use container::monitor::{Monitor};
use container::monitor::{Killed, Exit};
use container::container::{Command};
use super::Wrapper;
use super::util::find_cmd;
use super::setup;


pub fn supervise_cmd(cname: &String, command: &SuperviseInfo,
    wrapper: &Wrapper, cmdline: Vec<String>)
    -> Result<int, String>
{
    let mut child = "".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Runs a single command from supervision suite");
        ap.refer(&mut child)
            .add_argument("child", box Store::<String>,
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
    try!(setup::setup_base_filesystem(
        wrapper.project_root, wrapper.ext_settings));

    let childtype = try!(command.children.find(&child)
        .ok_or(format!("Child {} not found", child)));
    match childtype {
        &Command(ref info) => supervise_child_command(cname,
            &child, false, info, wrapper, command),
        &BridgeCommand(ref info) => supervise_child_command(cname,
            &child, true, info, wrapper, command),
    }
}

fn _write_hosts(supervise: &SuperviseInfo) -> Result<(), String> {
    let basedir = Path::new("/tmp/vagga");
    if !basedir.is_dir() {
        try!(mkdir(&basedir, ALL_PERMISSIONS)
            .map_err(|e| format!("Can't create dir: {}", e)));
    }
    let mut file = File::create(&basedir.join("hosts"));
    try!((writeln!(file, "127.0.0.1 localhost"))
         .map_err(|e| format!("Error writing hosts: {}", e)));
    for (subname, subcommand) in supervise.children.iter() {
        if let &Command(ref cmd) = subcommand {
            if let Some(ref netw) = cmd.network {
                // TODO(tailhook) support multiple commands with same IP
                if netw.hostname.is_none() {
                    try!((writeln!(file, "{} {}", netw.ip, subname))
                         .map_err(|e| format!("Error writing hosts: {}", e)));
                } else {
                    try!((writeln!(file, "{} {} {}", netw.ip,
                                    netw.hostname, subname))
                         .map_err(|e| format!("Error writing hosts: {}", e)));
                }
            }
        }
    }
    return Ok(());
}

fn supervise_child_command(cmdname: &String, name: &String, bridge: bool,
    command: &CommandInfo, wrapper: &Wrapper, supervise: &SuperviseInfo)
    -> Result<int, String>
{
    let cconfig = try!(wrapper.config.containers.find(&command.container)
        .ok_or(format!("Container {} not found", command.container)));
    let uid_map = try!(map_users(wrapper.settings,
        &cconfig.uids, &cconfig.gids));

    let lnk = try!(readlink(&Path::new("/work/.vagga")
                   .join(command.container.as_slice()))
        .map_err(|e| format!("Error reading link: {}", e)));
    let lnkcmp = lnk.str_components().collect::<Vec<Option<&str>>>();
    if lnkcmp.len() < 3 || lnkcmp[lnkcmp.len()-2].is_none() {
        return Err(format!("Broken container link"));
    }

    try!(setup::setup_filesystem(cconfig, lnkcmp[lnkcmp.len()-2].unwrap()));

    try!(_write_hosts(supervise));

    let mut env = try!(setup::get_environment(cconfig));
    for (k, v) in command.environ.iter() {
        env.insert(k.clone(), v.clone());
    }
    let mut cmdline = command.run.clone();
    let cpath = try!(find_cmd(cmdline.remove(0).unwrap().as_slice(), &env));

    let mut cmd = Command::new(name.to_string(), &cpath);
    cmd.args(cmdline.as_slice());
    cmd.set_env("VAGGA_COMMAND".to_string(), cmdname.to_string());
    cmd.set_env("VAGGA_SUBCOMMAND".to_string(), name.to_string());
    if !bridge {
        cmd.set_uidmap(uid_map.clone());
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
