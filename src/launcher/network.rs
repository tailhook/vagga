use std::io::{stdout, stderr};
use std::io::fs::File;
use std::io::process::{Command, Ignored, InheritFd, ExitStatus};

use argparse::{ArgumentParser, StoreTrue};

use config::Config;


pub fn create_netns(config: &Config, mut args: Vec<String>)
    -> Result<int, String>
{
    let netns_name = "vagga".to_string();
    let interface_name = "vagga".to_string();
    let network = "172.18.0.0/30".to_string();
    let host_ip_net = "172.18.0.1/30".to_string();
    let host_ip = "172.18.0.1".to_string();
    let guest_ip = "172.18.0.2/30".to_string();
    let mut dry_run = false;
    let mut iptables = true;
    {
        args.insert(0, "vagga _create_netns".to_string());
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Set's up network namespace for subsequent container runs
            ");
        ap.refer(&mut dry_run)
            .add_option(&["--dry-run"], box StoreTrue,
                "Do not run commands, only show");
        ap.refer(&mut dry_run)
            .add_option(&["--no-iptables"], box StoreTrue,
                "Do not update iptables rules (useful you have firewall \
                 other than iptables). You need to update your firewall rules \
                 manually to have functional networking.");
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => {
                return Ok(122);
            }
        }
    }
    println!("We will run network setup commands with sudo.");
    println!("You may need to enter your password.");

    let mut commands = vec!();

    // If we are root we may skip sudo
    let mut ip_cmd = Command::new("sudo");
    ip_cmd.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2));
    ip_cmd.arg("ip");

    let mut cmd = ip_cmd.clone();
    cmd.args(["netns", "add", netns_name.as_slice()]);
    commands.push(cmd);

    let mut ns_cmd = ip_cmd.clone();
    ns_cmd.args(["netns", "exec", netns_name.as_slice()]);

    let mut cmd = ns_cmd.clone();
    cmd.args(["ip", "link", "set", "dev", "lo", "up"]);
    commands.push(cmd);

    let mut cmd = ns_cmd.clone();
    cmd.args(["ip", "link", "add", "vagga_guest", "type", "veth",
              "peer", "name", interface_name.as_slice()]);
    commands.push(cmd);

    let mut cmd = ns_cmd.clone();
    cmd.args(["ip", "link", "set", "dev", "vagga_guest", "up"]);
    commands.push(cmd);

    let mut cmd = ns_cmd.clone();
    cmd.args(["ip", "link", "set", interface_name.as_slice(), "netns", "1"]);
    commands.push(cmd);

    let mut cmd = ip_cmd.clone();
    cmd.args(["addr", "add", host_ip_net.as_slice(),
              "dev", interface_name.as_slice()]);
    commands.push(cmd);

    let mut cmd = ns_cmd.clone();
    cmd.args(["ip", "addr", "add", guest_ip.as_slice(), "dev", "vagga_guest"]);
    commands.push(cmd);

    let mut cmd = ns_cmd.clone();
    cmd.args(["ip", "route", "add", "default", "via", host_ip.as_slice()]);
    commands.push(cmd);

    let nforward = try!(File::open(&Path::new("/proc/sys/net/ipv4/ip_forward"))
        .and_then(|mut f| f.read_to_string())
        .map_err(|e| format!("Can't read sysctl: {}", e)));

    if nforward.as_slice().trim() == "0" {
        // If we are root we may skip sudo
        let mut cmd = Command::new("sudo");
        cmd.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2));
        cmd.args(["sysctl", "net.ipv4.ip_forward=1"]);
        commands.push(cmd);
    } else {
        info!("Sysctl is ok [{}]", nforward.as_slice().trim());
    }

    println!("");
    println!("The following commands will be run:");
    for cmd in commands.iter() {
        println!("    {}", cmd);
    }

    if !dry_run {
        for cmd in commands.iter() {
            match cmd.status() {
                Ok(ExitStatus(0)) => {},
                val => return Err(
                    format!("Error running command {}: {}", cmd, val)),
            }
        }
    }

    if iptables {
        println!("");
        println!("Checking firewall rules:");
        let mut iptables = Command::new("sudo");
        iptables.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2));
        iptables.arg("iptables");

        let mut cmd = iptables.clone();
        iptables.stderr(InheritFd(1));  // Message not an error actually
        cmd.args(["-t", "nat", "-C", "POSTROUTING",
                  "-s", network.as_slice(), "-j", "MASQUERADE"]);
        println!("    {}", cmd);
        let exists = match cmd.status() {
            Ok(ExitStatus(0)) => true,
            Ok(ExitStatus(1)) => false,
            val => return Err(
                format!("Error running command {}: {}", cmd, val)),
        };

        if exists {
            println!("Already setup. Skipping...");
        } else {
            let mut cmd = iptables.clone();
            cmd.args(["-t", "nat", "-A", "POSTROUTING",
                      "-s", network.as_slice(), "-j", "MASQUERADE"]);
            println!("Not existent, creating:");
            println!("    {}", cmd);
            match cmd.status() {
                Ok(ExitStatus(0)) => {},
                val => return Err(
                    format!("Error setting up iptables {}: {}", cmd, val)),
            }
        }
    }

    Ok(0)
}
