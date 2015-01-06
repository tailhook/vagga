use std::rc::Rc;
use std::os::{getenv, self_exe_path};
use std::rand::random;
use std::io::process::{Command, Ignored, InheritFd, ExitStatus};
use libc::c_int;

use super::run::DEFAULT_PATH;
use container::container::Command as ContainerCommand;
use container::util::nsopen;
use container::mount::{mount_pseudo};
use container::monitor::{Monitor, RunOnce, Exit, Killed};


fn _run_command(cmd: Command) -> Result<(), String> {
    debug!("Running {}", cmd);
    match cmd.status() {
        Ok(ExitStatus(0)) => Ok(()),
        code => Err(format!("Error running {}: {}",  cmd, code)),
    }
}

pub fn setup_ip_address(ip: String) -> Result<c_int, String> {

    // Must use iproute2 from host system because busybox doesn't support veth
    let mut ip_cmd = Command::new("ip");
    ip_cmd.env("PATH", getenv("HOST_PATH").unwrap_or(DEFAULT_PATH.to_string()));
    ip_cmd.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2));

    let id: u16 = random();
    let eif = format!("ve{}", id);
    let iif = format!("ve{}c", id);

    let mut cmd = ip_cmd.clone();
    cmd.args(["link", "add", eif.as_slice(), "type", "veth",
              "peer", "name", iif.as_slice()]);
    try!(_run_command(cmd));

    let mut cmd = ip_cmd.clone();
    cmd.args(["link", "set", "dev", eif.as_slice(), "up"]);
    try!(_run_command(cmd));

    let mut cmd = ip_cmd.clone();
    cmd.args(["link", "set", eif.as_slice(), "master", "children"]);
    try!(_run_command(cmd));

    let mut cmd = ip_cmd.clone();
    cmd.args(["addr", "add", ip.as_slice(), "dev", iif.as_slice()]);
    try!(_run_command(cmd));

    //  Unfortunately we are already in new namespace but do not have
    //  new procfs mounted yet
    //  This is hacky, but we discard root filesystem shortly so don't care
    //  TODO(tailhook) still find a better place/solution
    try!(mount_pseudo(&Path::new("/proc"), "proc", "", false));

    let cmdname = Rc::new("setup_netns".to_string());
    let mut mon = Monitor::new();
    let mut cmd = ContainerCommand::new(cmdname.to_string(),
        self_exe_path().unwrap().join("vagga_setup_netns"));
    cmd.args(["container", "--interface", iif.as_slice()]);
    cmd.network_ns();
    mon.add(cmdname.clone(), box RunOnce::new(cmd));
    let pid = try!(mon.force_start(cmdname));

    let ns = try!(nsopen(pid, "net")
        .map_err(|e| format!("Error opening network namespace: {}", e)));

    let mut cmd = ip_cmd.clone();
    cmd.args(["link", "set", "dev", iif.as_slice(),
              "netns", format!("{}", pid).as_slice()]);
    try!(_run_command(cmd));

    match mon.run() {
        Exit(0) => {}
        Killed => return Err(format!("vagga_setup_netns is dead")),
        Exit(c) => return Err(
            format!("vagga_setup_netns exited with code: {}", c)),
    }
    return Ok(ns);
}
