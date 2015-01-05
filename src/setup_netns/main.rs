#![feature(phase, if_let)]

extern crate argparse;
#[phase(plugin, link)] extern crate log;

use std::io::BufferedReader;
use std::os::set_exit_status;
use std::os::self_exe_path;
use std::io::fs::File;
use std::io::timer::sleep;
use std::io::stdio::{stdout, stderr};
use std::time::duration::Duration;
use std::io::process::{Command, Ignored, InheritFd, ExitStatus};

use argparse::{ArgumentParser, Store, List};

fn has_interface() -> Result<bool, String> {
    File::open(&Path::new("/proc/net/dev"))
        .map(BufferedReader::new)
        .and_then(|mut f| {
            let mut lineiter = f.lines();
            try!(lineiter.next().unwrap());  // Two header lines
            try!(lineiter.next().unwrap());
            for line in lineiter {
                let line = try!(line);
                let mut splits = line.as_slice().splitn(1, ':');
                let interface = splits.next().unwrap();
                if interface == "vagga_guest" {
                    return Ok(true);
                }
            }
            return Ok(false);
        })
        .map_err(|e| format!("Can't read interfaces: {}", e))
}

fn setup_bridge_namespace(args: Vec<String>) {
    let mut guest_ip = "".to_string();
    let mut gateway_ip = "".to_string();
    let mut network = "".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Set up intermediate (bridge) network namespace
            ");
        ap.refer(&mut guest_ip)
            .add_option(&["--guest-ip"], box Store::<String>,
                "IP to use on the vagga_guest interface");
        ap.refer(&mut network)
            .add_option(&["--network"], box Store::<String>,
                "Network address");
        ap.refer(&mut gateway_ip)
            .add_option(&["--gateway-ip"], box Store::<String>,
                "Gateway address");
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return,
            Err(x) => {
                set_exit_status(x);
                return;
            }
        }
    }
    loop {
        match has_interface() {
            Ok(true) => break,
            Ok(false) => {}
            Err(x) => {
                error!("Error setting interface vagga_guest: {}", x);
                set_exit_status(1);
                return;
            }
        }
        sleep(Duration::milliseconds(100));
    }

    let busybox = Command::new(self_exe_path().unwrap().join("busybox"));

    let mut ip_cmd = busybox.clone();
    ip_cmd.arg("ip");
    ip_cmd.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2));

    let mut commands = vec!();

    let mut cmd = busybox.clone();
    cmd.args(&["brctl", "addbr", "children"]);
    commands.push(cmd);

    let mut cmd = ip_cmd.clone();
    cmd.args(&["addr", "add", guest_ip.as_slice(), "dev", "vagga_guest"]);
    commands.push(cmd);

    let mut cmd = ip_cmd.clone();
    cmd.args(&["link", "set", "dev", "vagga_guest", "up"]);
    commands.push(cmd);

    let mut cmd = ip_cmd.clone();
    cmd.args(&["link", "set", "dev", "lo", "up"]);
    commands.push(cmd);

    let mut cmd = ip_cmd.clone();
    cmd.args(&["route", "add", "default", "via", gateway_ip.as_slice()]);
    commands.push(cmd);

    let mut cmd = ip_cmd.clone();
    cmd.args(&["addr", "add", "172.18.0.254/24", "dev", "children"]);
    commands.push(cmd);

    // Unfortunately there is no iptables in busybox so use iptables from host
    let mut cmd = Command::new("iptables");
    cmd.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2));
    cmd.args(["-t", "nat", "-A", "POSTROUTING",
              "-s", "172.18.0.0/24", "-j", "MASQUERADE"]);
    commands.push(cmd);

    for cmd in commands.iter() {
        debug!("Running {}", cmd);
        match cmd.status() {
            Ok(ExitStatus(0)) => {},
            err => {
                error!("Error running command {}: {}", cmd, err);
                set_exit_status(1);
                return;
            }
        };
    }
}

fn main() {
    let mut kind = "".to_string();
    let mut args: Vec<String> = vec!();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Set up network namespace for containers
            ");
        ap.refer(&mut kind)
            .add_argument("kind", box Store::<String>,
                "Kind of namespace to set up (bridge or container)");
        ap.refer(&mut args)
            .add_argument("options", box List::<String>,
                "Options specific for this kind");
        ap.stop_on_first_argument(true);
        match ap.parse_args() {
            Ok(()) => {}
            Err(0) => return,
            Err(x) => {
                set_exit_status(x);
                return;
            }
        }
    }
    args.insert(0, format!("vagga_setup_netns {}", kind));
    match kind.as_slice() {
        "bridge" => setup_bridge_namespace(args),
        _ => {
            set_exit_status(1);
            error!("Unknown command {}", kind);
        }
    }
}
