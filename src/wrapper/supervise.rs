use std::io::fs::{readlink};
use std::os::getenv;
use std::io::stdio::{stdout, stderr};

use argparse::{ArgumentParser, Store};

use config::command::{SuperviseInfo, ChildCommandInfo};
use config::command::child::Command;
use container::uidmap::{map_users};
use container::monitor::{Monitor};
use container::monitor::{Killed, Exit};
use container::container::{Command};
use super::Wrapper;
use super::util::find_cmd;
use super::setup;


pub fn supervise_cmd(command: &SuperviseInfo, wrapper: &Wrapper,
    cmdline: Vec<String>)
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
        &Command(ref info) => supervise_child_command(
            &child, info, wrapper, command),
    }
}

fn supervise_child_command(name: &String, command: &ChildCommandInfo,
    wrapper: &Wrapper, _supervise: &SuperviseInfo)
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

    let mut env = try!(setup::get_environment(cconfig));
    for (k, v) in command.environ.iter() {
        env.insert(k.clone(), v.clone());
    }
    let mut cmdline = command.run.clone();
    let cpath = try!(find_cmd(cmdline.remove(0).unwrap().as_slice(), &env));

    let mut cmd = Command::new(name.to_string(), &cpath);
    cmd.args(cmdline.as_slice());
    cmd.set_uidmap(uid_map.clone());
    if command.network.ip.is_some() {  // TODO(tailhook) network Option'al
        cmd.set_network(command.network.clone());
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
