use std::rc::Rc;
use std::os::{getenv, self_exe_path};
use std::rand::random;
use std::io::process::{Command, Ignored, InheritFd, ExitStatus};

use super::run::DEFAULT_PATH;
use container::mount::{mount_pseudo};
use container::nsutil::set_namespace;
use container::container::Command as ContainerCommand;
use container::container::NewNet;
use container::monitor::{Monitor, RunOnce, Exit, Killed};


fn _run_command(cmd: Command) -> Result<(), String> {
    debug!("Running {}", cmd);
    match cmd.status() {
        Ok(ExitStatus(0)) => Ok(()),
        code => Err(format!("Error running {}: {}",  cmd, code)),
    }
}

pub fn setup_ip_address(ip: String) -> Result<(), String> {

    //  Must use iproute2 from host system because busybox doesn't support veth
    //  User need to have iproute2 anyway to run _create_netns/_destroy_netns
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

    //  Use busybox for brctl so user don't need
    let mut busybox = Command::new(self_exe_path().unwrap().join("busybox"));
    busybox.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2));

    let mut cmd = busybox.clone();
    cmd.args(["brctl", "addif", "children", eif.as_slice()]);
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
    cmd.args(["container", "--interface", iif.as_slice(),
                           "--ip", ip.as_slice(),
                           "--gateway", "172.18.0.254"]);
    cmd.network_ns();
    cmd.set_env("TERM".to_string(),
                getenv("TERM").unwrap_or("dumb".to_string()));
    if let Some(x) = getenv("RUST_LOG") {
        cmd.set_env("RUST_LOG".to_string(), x);
    }
    if let Some(x) = getenv("RUST_BACKTRACE") {
        cmd.set_env("RUST_BACKTRACE".to_string(), x);
    }

    mon.add(cmdname.clone(), box RunOnce::new(cmd));
    let pid = try!(mon.force_start(cmdname));

    let mut cmd = ip_cmd.clone();
    cmd.args(["addr"]);
    try!(_run_command(cmd));

    let mut cmd = ip_cmd.clone();
    cmd.args(["link", "set", "dev", iif.as_slice(),
              "netns", format!("{}", pid).as_slice()]);
    try!(_run_command(cmd));

    try!(set_namespace(
         &Path::new("/proc").join(format!("{}", pid)).join("ns/net"), NewNet)
        .map_err(|e| format!("Open network namespace: {}", e)));

    match mon.run() {
        Exit(0) => {}
        Killed => return Err(format!("vagga_setup_netns is dead")),
        Exit(c) => return Err(
            format!("vagga_setup_netns exited with code: {}", c)),
    }
    Ok(())
}
