use std::io::ALL_PERMISSIONS;
use std::io::fs::{mkdir, readlink};
use std::io::fs::PathExtensions;
use std::os::getenv;
use std::io::stdio::{stdout, stderr};

use argparse::{ArgumentParser, Store};

use config::command::{SuperviseInfo, ChildCommandInfo};
use config::command::child::Command;
use container::root::change_root;
use container::mount::{bind_mount, unmount, mount_system_dirs, remount_ro};
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
    let childtype = try!(command.children.find(&child)
        .ok_or(format!("Child {} not found", child)));
    match childtype {
        &Command(ref info) => supervise_child_command(info, wrapper, command),
    }
}

fn supervise_child_command(command: &ChildCommandInfo, wrapper: &Wrapper,
    _supervise: &SuperviseInfo)
    -> Result<int, String>
{
    let cconfig = try!(wrapper.config.containers.find(&command.container)
        .ok_or(format!("Container {} not found", command.container)));
    let uid_map = try!(map_users(wrapper.settings,
        &cconfig.uids, &cconfig.gids));

    let tgtroot = Path::new("/vagga/root");
    if !tgtroot.exists() {
        try!(mkdir(&tgtroot, ALL_PERMISSIONS)
             .map_err(|x| format!("Error creating directory: {}", x)));
    }
    let lnk = try!(readlink(&Path::new("/work/.vagga")
                   .join(command.container.as_slice()))
        .map_err(|e| format!("Error reading link: {}", e)));
    let lnkcmp = lnk.str_components().collect::<Vec<Option<&str>>>();
    if lnkcmp.len() < 3 || lnkcmp[lnkcmp.len()-2].is_none() {
        return Err(format!("Broken container link"));
    }
    try!(bind_mount(&Path::new("/vagga/roots")
                    .join(lnkcmp[lnkcmp.len()-2].unwrap()).join("root"),
                    &tgtroot)
         .map_err(|e| format!("Error bind mount: {}", e)));
    try!(remount_ro(&tgtroot));
    try!(mount_system_dirs()
        .map_err(|e| format!("Error mounting system dirs: {}", e)));
    try!(change_root(&tgtroot, &tgtroot.join("tmp"))
         .map_err(|e| format!("Error changing root: {}", e)));
    try!(unmount(&Path::new("/work/.vagga/.mnt"))
         .map_err(|e| format!("Error unmounting `.vagga/.mnt`: {}", e)));
    try!(unmount(&Path::new("/tmp"))
         .map_err(|e| format!("Error unmounting old root: {}", e)));

    let mut env = try!(setup::get_environment(cconfig));
    for (k, v) in command.environ.iter() {
        env.insert(k.clone(), v.clone());
    }
    let mut cmdline = command.run.clone();
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
