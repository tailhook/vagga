use std::env;
use std::fs::File;
use std::io::{stdout, stderr};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::exit;
use std::process::{Command, Stdio};
use std::thread::sleep_ms;

use rustc_serialize::json;

use argparse::{ArgumentParser, Store, List};

fn has_interface(name: &str) -> Result<bool, String> {
    File::open(&Path::new("/proc/net/dev"))
        .map(BufReader::new)
        .and_then(|mut f| {
            let mut lineiter = f.lines();
            try!(lineiter.next().unwrap());  // Two header lines
            try!(lineiter.next().unwrap());
            for line in lineiter {
                let line = try!(line);
                let mut splits = line[..].splitn(1, ':');
                let interface = splits.next().unwrap();
                if interface.trim() == name {
                    return Ok(true);
                }
            }
            return Ok(false);
        })
        .map_err(|e| format!("Can't read interfaces: {}", e))
}

fn busybox_cmd() -> Command {
    let mut busybox = Command::new(
        env::current_exe().unwrap()
        .parent().unwrap()
        .join("busybox"));
    busybox.stdin(Stdio::null());
    busybox.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    busybox
}

fn ip_cmd() -> Command {
    let mut ip_cmd = busybox_cmd();
    ip_cmd.arg("ip");
    ip_cmd.stdin(Stdio::null());
    ip_cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    ip_cmd
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
                exit(x);
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
                exit(1);
                return;
            }
        }
        sleep_ms(100);
    }


    let mut commands = vec!();

    let mut cmd = ip_cmd();
    cmd.args(&["link", "set", "dev", "lo", "up"]);
    commands.push(cmd);


    let mut cmd = ip_cmd();
    cmd.args(&["addr", "add", &guest_ip[..], "dev", "vagga_guest"]);
    commands.push(cmd);

    let mut cmd = ip_cmd();
    cmd.args(&["link", "set", "dev", "vagga_guest", "up"]);
    commands.push(cmd);

    let mut cmd = ip_cmd();
    cmd.args(&["route", "add", "default", "via", &gateway_ip[..]]);
    commands.push(cmd);

    // Unfortunately there is no iptables in busybox so use iptables from host
    let mut cmd = Command::new("iptables");
    cmd.stdin(Stdio::null()).stdout(Stdio::inherit()).stderr(Stdio::inherit());
    cmd.args(&["-t", "nat", "-A", "POSTROUTING",
               "-o", "vagga_guest",
               "-j", "MASQUERADE"]);
    commands.push(cmd);

    for mut cmd in commands.into_iter() {
        debug!("Running {:?}", cmd);
        match cmd.status() {
            Ok(status) if status.success() => {},
            err => {
                error!("Error running command {:?}: {:?}", cmd, err);
                exit(1);
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
                exit(x);
                return;
            }
        }
    }
    let ports: Vec<(u16, String, u16)> = json::decode(&ports_str)
        .ok().expect("Port-forwards JSON is invalid");
    loop {
        match has_interface(&interface) {
            Ok(true) => break,
            Ok(false) => {}
            Err(x) => {
                error!("Error setting interface {}: {}", interface, x);
                exit(1);
                return;
            }
        }
        sleep_ms(100);
    }
    let mut commands = vec!();

    let mut cmd = ip_cmd();
    cmd.args(&["link", "set", "dev", "lo", "up"]);
    commands.push(cmd);

    let mut cmd = ip_cmd();
    cmd.args(&["addr", "add", &format!("{}/30", ip)[..],
                       "dev", &interface]);
    commands.push(cmd);

    let mut cmd = ip_cmd();
    cmd.args(&["link", "set", "dev", &interface[..], "up"]);
    commands.push(cmd);

    let mut cmd = ip_cmd();
    cmd.args(&["route", "add", "default", "via", &gateway_ip[..]]);
    commands.push(cmd);

    let mut cmd = busybox_cmd();
    cmd.args(&["brctl", "addbr", "children"]);
    commands.push(cmd);

    let mut cmd = ip_cmd();
    cmd.args(&["addr", "add", "172.18.0.254/24", "dev", "children"]);
    commands.push(cmd);

    let mut cmd = ip_cmd();
    cmd.args(&["link", "set", "dev", "children", "up"]);
    commands.push(cmd);

    // Unfortunately there is no iptables in busybox so use iptables from host
    let mut cmd = Command::new("iptables");
    cmd.stdin(Stdio::null()).stdout(Stdio::inherit()).stderr(Stdio::inherit());
    cmd.args(&["-t", "nat", "-A", "POSTROUTING",
               "-s", "172.18.0.0/24", "-j", "MASQUERADE"]);
    commands.push(cmd);

    for &(sport, ref dip, dport) in ports.iter() {
        let mut cmd = Command::new("iptables");
        cmd.stdin(Stdio::null()).stdout(Stdio::inherit()).stderr(Stdio::inherit());
        cmd.args(&["-t", "nat", "-A", "PREROUTING", "-p", "tcp", "-m", "tcp",
            "--dport", &format!("{}", sport)[..], "-j", "DNAT",
            "--to-destination", &format!("{}:{}", dip, dport)[..]]);
        commands.push(cmd);
    }

    for mut cmd in commands.into_iter() {
        debug!("Running {:?}", cmd);
        match cmd.status() {
            Ok(status) if status.success() => {}
            err => {
                error!("Error running command {:?}: {:?}", cmd, err);
                exit(1);
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
                exit(x);
                return;
            }
        }
    }
    loop {
        match has_interface(&interface) {
            Ok(true) => break,
            Ok(false) => {}
            Err(x) => {
                error!("Error setting interface {}: {}", interface, x);
                exit(1);
                return;
            }
        }
        sleep_ms(100);
    }
    let mut commands = vec!();

    let mut cmd = ip_cmd();
    cmd.args(&["link", "set", "dev", "lo", "up"]);
    commands.push(cmd);

    let mut cmd = ip_cmd();
    cmd.args(&["addr", "add", &format!("{}/24", ip)[..],
                       "dev", &interface[..]]);
    commands.push(cmd);

    let mut cmd = ip_cmd();
    cmd.args(&["link", "set", "dev", &interface[..], "up"]);
    commands.push(cmd);

    let mut cmd = busybox_cmd();
    cmd.args(&["hostname", &hostname[..]]);
    commands.push(cmd);

    let mut cmd = ip_cmd();
    cmd.args(&["route", "add", "default", "via", &gateway_ip[..]]);
    commands.push(cmd);

    for mut cmd in commands.into_iter() {
        debug!("Running {:?}", cmd);
        match cmd.status() {
            Ok(status) if status.success() => {}
            err => {
                error!("Error running command {:?}: {:?}", cmd, err);
                exit(1);
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
        ap.parse_args_or_exit();
    }
    args.insert(0, format!("vagga_setup_netns {}", kind));
    match &kind[..] {
        "gateway" => setup_gateway_namespace(args),
        "bridge" => setup_bridge_namespace(args),
        "guest" => setup_guest_namespace(args),
        _ => {
            exit(1);
            error!("Unknown command {}", kind);
        }
    }
}
