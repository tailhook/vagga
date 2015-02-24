extern crate argparse;
extern crate serialize;
#[macro_use] extern crate log;

use std::os::env;
use std::io::BufferedReader;
use std::os::set_exit_status;
use std::os::self_exe_path;
use std::io::fs::File;
use std::io::timer::sleep;
use std::io::stdio::{stdout, stderr};
use std::time::duration::Duration;
use std::io::process::{Command, Ignored, InheritFd, ExitStatus};
use serialize::json;

use argparse::{ArgumentParser, Store, List};

fn has_interface(name: &str) -> Result<bool, String> {
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
                if interface.trim() == name {
                    return Ok(true);
                }
            }
            return Ok(false);
        })
        .map_err(|e| format!("Can't read interfaces: {}", e))
}

fn setup_gateway_namespace(args: Vec<String>) {
    let mut guest_ip = "".to_string();
    let mut gateway_ip = "".to_string();
    let mut network = "".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Set up intermediate (gateway) network namespace
            ");
        ap.refer(&mut guest_ip)
            .add_option(&["--guest-ip"], Store,
                "IP to use on the vagga_guest interface");
        ap.refer(&mut network)
            .add_option(&["--network"], Store,
                "Network address");
        ap.refer(&mut gateway_ip)
            .add_option(&["--gateway-ip"], Store,
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
        match has_interface("vagga_guest") {
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

    let mut busybox = Command::new(self_exe_path().unwrap().join("busybox"));
    busybox.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2));

    let mut ip_cmd = busybox.clone();
    ip_cmd.arg("ip");
    ip_cmd.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2));

    let mut commands = vec!();

    let mut cmd = ip_cmd.clone();
    cmd.args(&["link", "set", "dev", "lo", "up"]);
    commands.push(cmd);


    let mut cmd = ip_cmd.clone();
    cmd.args(&["addr", "add", guest_ip.as_slice(), "dev", "vagga_guest"]);
    commands.push(cmd);

    let mut cmd = ip_cmd.clone();
    cmd.args(&["link", "set", "dev", "vagga_guest", "up"]);
    commands.push(cmd);

    let mut cmd = ip_cmd.clone();
    cmd.args(&["route", "add", "default", "via", gateway_ip.as_slice()]);
    commands.push(cmd);

    // Unfortunately there is no iptables in busybox so use iptables from host
    let mut cmd = Command::new("iptables");
    cmd.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2));
    cmd.args(&["-t", "nat", "-A", "POSTROUTING",
               "-o", "vagga_guest",
               "-j", "MASQUERADE"]);
    commands.push(cmd);

    for cmd in commands.iter() {
        debug!("Running {}", cmd);
        match cmd.status() {
            Ok(ExitStatus(0)) => {},
            err => {
                error!("Error running command {}: {:?}", cmd, err);
                set_exit_status(1);
                return;
            }
        };
    }
}

fn setup_bridge_namespace(args: Vec<String>) {
    let mut interface = "".to_string();
    let mut ip = "".to_string();
    let mut gateway_ip = "".to_string();
    let mut ports_str = "".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Set up intermediate (bridge) network namespace
            ");
        ap.refer(&mut interface)
            .add_option(&["--interface"], Store,
                "Network interface name")
            .required();
        ap.refer(&mut ip)
            .add_option(&["--ip"], Store,
                "IP to use on the interface")
            .required();
        ap.refer(&mut gateway_ip)
            .add_option(&["--gateway-ip"], Store,
                "Gateway to use on the interface")
            .required();
        ap.refer(&mut ports_str)
            .add_option(&["--port-forwards"], Store,
                "Port forwards though bridge")
            .required();
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return,
            Err(x) => {
                set_exit_status(x);
                return;
            }
        }
    }
    let ports: Vec<(u16, String, u16)> = json::decode(ports_str.as_slice())
        .ok().expect("Port-forwards JSON is invalid");
    loop {
        match has_interface(interface.as_slice()) {
            Ok(true) => break,
            Ok(false) => {}
            Err(x) => {
                error!("Error setting interface {}: {}", interface, x);
                set_exit_status(1);
                return;
            }
        }
        sleep(Duration::milliseconds(100));
    }
    let mut commands = vec!();

    let busybox = Command::new(self_exe_path().unwrap().join("busybox"));

    let mut ip_cmd = busybox.clone();
    ip_cmd.arg("ip");
    ip_cmd.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2));

    let mut cmd = ip_cmd.clone();
    cmd.args(&["link", "set", "dev", "lo", "up"]);
    commands.push(cmd);

    let mut cmd = ip_cmd.clone();
    cmd.args(&["addr", "add", format!("{}/30", ip).as_slice(),
                       "dev", interface.as_slice()]);
    commands.push(cmd);

    let mut cmd = ip_cmd.clone();
    cmd.args(&["link", "set", "dev", interface.as_slice(), "up"]);
    commands.push(cmd);

    let mut cmd = ip_cmd.clone();
    cmd.args(&["route", "add", "default", "via", gateway_ip.as_slice()]);
    commands.push(cmd);

    let mut cmd = busybox.clone();
    cmd.args(&["brctl", "addbr", "children"]);
    commands.push(cmd);

    let mut cmd = ip_cmd.clone();
    cmd.args(&["addr", "add", "172.18.0.254/24", "dev", "children"]);
    commands.push(cmd);

    let mut cmd = ip_cmd.clone();
    cmd.args(&["link", "set", "dev", "children", "up"]);
    commands.push(cmd);

    // Unfortunately there is no iptables in busybox so use iptables from host
    let mut cmd = Command::new("iptables");
    cmd.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2));
    cmd.args(&["-t", "nat", "-A", "POSTROUTING",
               "-s", "172.18.0.0/24", "-j", "MASQUERADE"]);
    commands.push(cmd);

    for &(sport, ref dip, dport) in ports.iter() {
        let mut cmd = Command::new("iptables");
        cmd.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2));
        cmd.args(&["-t", "nat", "-A", "PREROUTING", "-p", "tcp", "-m", "tcp",
            "--dport", format!("{}", sport).as_slice(), "-j", "DNAT",
            "--to-destination", format!("{}:{}", dip, dport).as_slice()]);
        commands.push(cmd);
    }

    for cmd in commands.iter() {
        debug!("Running {}", cmd);
        match cmd.status() {
            Ok(ExitStatus(0)) => {},
            err => {
                error!("Error running command {}: {:?}", cmd, err);
                set_exit_status(1);
                return;
            }
        };
    }
}

fn setup_guest_namespace(args: Vec<String>) {
    let mut interface = "".to_string();
    let mut ip = "".to_string();
    let mut gateway_ip = "".to_string();
    let mut hostname = "".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Set up guest network namespace
            ");
        ap.refer(&mut interface)
            .add_option(&["--interface"], Store,
                "Network interface name")
            .required();
        ap.refer(&mut ip)
            .add_option(&["--ip"], Store,
                "IP to use on the interface")
            .required();
        ap.refer(&mut gateway_ip)
            .add_option(&["--gateway-ip"], Store,
                "Gateway to use on the interface")
            .required();
        ap.refer(&mut hostname)
            .add_option(&["--hostname"], Store,
                "IP and hostname to use")
            .required();
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
        match has_interface(interface.as_slice()) {
            Ok(true) => break,
            Ok(false) => {}
            Err(x) => {
                error!("Error setting interface {}: {}", interface, x);
                set_exit_status(1);
                return;
            }
        }
        sleep(Duration::milliseconds(100));
    }
    let mut commands = vec!();

    let mut busybox = Command::new(self_exe_path().unwrap().join("busybox"));
    busybox.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2));

    let mut ip_cmd = busybox.clone();
    ip_cmd.arg("ip");
    ip_cmd.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2));

    let mut cmd = ip_cmd.clone();
    cmd.args(&["link", "set", "dev", "lo", "up"]);
    commands.push(cmd);

    let mut cmd = ip_cmd.clone();
    cmd.args(&["addr", "add", format!("{}/24", ip).as_slice(),
                       "dev", interface.as_slice()]);
    commands.push(cmd);

    let mut cmd = ip_cmd.clone();
    cmd.args(&["link", "set", "dev", interface.as_slice(), "up"]);
    commands.push(cmd);

    let mut cmd = busybox.clone();
    cmd.args(&["hostname", hostname.as_slice()]);
    commands.push(cmd);

    let mut cmd = ip_cmd.clone();
    cmd.args(&["route", "add", "default", "via", gateway_ip.as_slice()]);
    commands.push(cmd);

    for cmd in commands.iter() {
        debug!("Running {}", cmd);
        match cmd.status() {
            Ok(ExitStatus(0)) => {},
            err => {
                error!("Error running command {}: {:?}", cmd, err);
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
            .add_argument("kind", Store,
                "Kind of namespace to set up (bridge or container)");
        ap.refer(&mut args)
            .add_argument("options", List,
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
        "gateway" => setup_gateway_namespace(args),
        "bridge" => setup_bridge_namespace(args),
        "guest" => setup_guest_namespace(args),
        _ => {
            set_exit_status(1);
            error!("Unknown command {}", kind);
        }
    }
}
