use std::io::process::{Command, Ignored, InheritFd, ExitStatus};
use std::rand::random;
use libc::pid_t;

use config::command::Network;


fn _run_command(cmd: Command) -> Result<(), String> {
    debug!("Running {}", cmd);
    match cmd.status() {
        Ok(ExitStatus(0)) => Ok(()),
        code => Err(format!("Error running {}: {}",  cmd, code)),
    }
}


pub fn apply_network(network: &Network, pid: pid_t) -> Result<(), String> {
    let ip = network.ip.as_ref().expect("Expected IP to be set");
    let mut bbox = Command::new("/tmp/vagga/bin/busybox");
    bbox.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2));

    let id: u16 = random();
    let eif = format!("ve{}", id);
    let iif = format!("ve{}c", id);

    let mut cmd = bbox.clone();
    cmd.args(["ip", "link", "add", eif.as_slice(), "type", "veth",
              "peer", "name", iif.as_slice()]);
    try!(_run_command(cmd));

    let mut cmd = bbox.clone();
    cmd.args(["ip", "link", "set", "dev", eif.as_slice(), "up"]);
    try!(_run_command(cmd));

    let mut cmd = bbox.clone();
    cmd.args(["ip", "link", "set", eif.as_slice(), "master", "children"]);
    try!(_run_command(cmd));

    let mut cmd = bbox.clone();
    cmd.args(["ip", "addr", "add", ip.as_slice(), "dev", iif.as_slice()]);
    try!(_run_command(cmd));

    return Ok(());
}
