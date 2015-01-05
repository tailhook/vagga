use std::rc::Rc;
use std::io::{stdout, stderr};
use std::os::{getenv, self_exe_path};
use std::io::{USER_RWX};
use std::io::{BufferedReader};
use std::io::fs::{File, PathExtensions};
use std::io::fs::{mkdir, unlink};
use std::io::process::{Command, Ignored, InheritFd, ExitStatus};
use libc::funcs::posix88::unistd::{geteuid};

use argparse::{ArgumentParser, StoreTrue, StoreFalse};

use config::Config;
use container::util::get_user_name;
use container::nsutil::set_namespace;
use container::container::{NewUser, NewNet};
use container::monitor::{Monitor, Exit, Killed, RunOnce};

use super::user;


pub fn namespace_dir() -> Path {
    let uid = unsafe { geteuid() };
    getenv("XDG_RUNTIME_DIR")
        .map(|v| Path::new(v).join("vagga"))
        .unwrap_or(Path::new(format!("/tmp/vagga-{}", get_user_name(uid))))
}


pub fn create_netns(_config: &Config, mut args: Vec<String>)
    -> Result<int, String>
{
    let interface_name = "vagga".to_string();
    let network = "172.18.255.0/30".to_string();
    let host_ip_net = "172.18.255.1/30".to_string();
    let host_ip = "172.18.255.1".to_string();
    let guest_ip = "172.18.255.2/30".to_string();
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
        ap.refer(&mut iptables)
            .add_option(&["--no-iptables"], box StoreFalse,
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

    let runtime_dir = namespace_dir();
    if !runtime_dir.exists() {
        try!(mkdir(&runtime_dir, USER_RWX)
            .map_err(|e| format!("Can't create runtime_dir: {}", e)));
    }

    let netns_file = runtime_dir.join("netns");
    let userns_file = runtime_dir.join("userns");

    if netns_file.exists() || userns_file.exists() {
        return Err("Namespaces already created".to_string());
    }

    let mut mon = Monitor::new();
    let vsn = Rc::new("vagga_setup_netns".to_string());
    {
        use container::container::Command;
        let mut cmd = Command::new("setup_netns".to_string(),
            self_exe_path().unwrap().join("vagga_setup_netns"));
        cmd.set_max_uidmap();
        cmd.network_ns();
        cmd.set_env("TERM".to_string(),
                    getenv("TERM").unwrap_or("dumb".to_string()));
        if let Some(x) = getenv("RUST_LOG") {
            cmd.set_env("RUST_LOG".to_string(), x);
        }
        if let Some(x) = getenv("RUST_BACKTRACE") {
            cmd.set_env("RUST_BACKTRACE".to_string(), x);
        }
        cmd.arg("bridge");
        cmd.arg("--guest-ip");
        cmd.arg(guest_ip.as_slice());
        cmd.arg("--gateway-ip");
        cmd.arg(host_ip.as_slice());
        cmd.arg("--network");
        cmd.arg(network.as_slice());
        mon.add(vsn.clone(), box RunOnce::new(cmd));
    }
    let child_pid = if dry_run { 123456 } else { try!(mon.force_start(vsn)) };

    println!("We will run network setup commands with sudo.");
    println!("You may need to enter your password.");

    let mut commands = vec!();

    // If we are root we may skip sudo
    let mut ip_cmd = Command::new("sudo");
    ip_cmd.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2));
    ip_cmd.arg("ip");

    // If we are root we may skip sudo
    let mut sysctl = Command::new("sudo");
    sysctl.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2));
    sysctl.arg("sysctl");

    let mut cmd = ip_cmd.clone();
    cmd.args(["link", "add", "vagga_guest", "type", "veth",
              "peer", "name", interface_name.as_slice()]);
    commands.push(cmd);

    let mut cmd = ip_cmd.clone();
    cmd.args(["link", "set", "vagga_guest", "netns"]);
    cmd.arg(format!("{}", child_pid));
    commands.push(cmd);

    let mut cmd = ip_cmd.clone();
    cmd.args(["addr", "add", host_ip_net.as_slice(),
              "dev", interface_name.as_slice()]);
    commands.push(cmd);

    let nforward = try!(File::open(&Path::new("/proc/sys/net/ipv4/ip_forward"))
        .and_then(|mut f| f.read_to_string())
        .map_err(|e| format!("Can't read sysctl: {}", e)));

    if nforward.as_slice().trim() == "0" {
        let mut cmd = sysctl.clone();
        cmd.args(["net.ipv4.ip_forward=1"]);
        commands.push(cmd);
    } else {
        info!("Sysctl is ok [{}]", nforward.as_slice().trim());
    }

    let mut cmd = sysctl.clone();
    cmd.args(["net.ipv4.conf.vagga.route_localnet=1"]);
    commands.push(cmd);

    let nameservers = try!(get_nameservers());
    info!("Detected nameservers: {}", nameservers);

    let local_dns = nameservers.as_slice() == ["127.0.0.1".to_string()];

    if !dry_run {
        try!(File::create(&netns_file)
            .map_err(|e| format!("Error creating netns file: {}", e)));
        try!(File::create(&userns_file)
            .map_err(|e| format!("Error creating userns file: {}", e)));
    }

    // If we are root we may skip sudo
    let mut mount_cmd = Command::new("sudo");
    mount_cmd.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2));
    mount_cmd.arg("mount");

    let mut cmd = mount_cmd.clone();
    cmd.arg("--bind");
    cmd.arg(format!("/proc/{}/ns/net", child_pid));
    cmd.arg(netns_file);
    commands.push(cmd);

    let mut cmd = mount_cmd.clone();
    cmd.arg("--bind");
    cmd.arg(format!("/proc/{}/ns/user", child_pid));
    cmd.arg(userns_file);
    commands.push(cmd);

    let mut iprules = vec!();
    if local_dns {
        iprules.push(vec!("-I", "INPUT",
                          "-i", interface_name.as_slice(),
                          "-d", "127.0.0.1",
                          "-j", "ACCEPT"));
        iprules.push(vec!("-t", "nat", "-I", "PREROUTING",
                          "-p", "tcp", "-i", "vagga",
                          "-d", host_ip.as_slice(), "--dport", "53",
                          "-j", "DNAT", "--to-destination", "127.0.0.1"));
        iprules.push(vec!("-t", "nat", "-I", "PREROUTING",
                          "-p", "udp", "-i", "vagga",
                          "-d", host_ip.as_slice(), "--dport", "53",
                          "-j", "DNAT", "--to-destination", "127.0.0.1"));
    }
    iprules.push(vec!("-t", "nat", "-A", "POSTROUTING",
                      "-s", network.as_slice(), "-j", "MASQUERADE"));


    println!("");
    println!("The following commands will be run:");
    for cmd in commands.iter() {
        println!("    {}", cmd);
    }



    if iptables {
        println!("");
        println!("The following iptables rules will be established:");

        for rule in iprules.iter() {
            println!("    {}", rule);
        }
    }

    if !dry_run {
        for cmd in commands.iter() {
            match cmd.status() {
                Ok(ExitStatus(0)) => {},
                val => return Err(
                    format!("Error running command {}: {}", cmd, val)),
            }
        }

        match mon.run() {
            Exit(0) => {}
            Killed => return Err(format!("vagga_setup_netns is dead")),
            Exit(c) => return Err(
                format!("vagga_setup_netns exited with code: {}", c)),
        }
        if iptables {
            let mut iptables = Command::new("sudo");
            iptables.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2));
            iptables.arg("iptables");

            for rule in iprules.iter() {
                let mut cmd = iptables.clone();
                iptables.stderr(InheritFd(1));  // Message not an error actually
                let mut check_rule = rule.clone();
                for item in check_rule.iter_mut() {
                    if *item == "-A" || *item == "-I" {
                        *item = "-C";
                    }
                }
                cmd.args(check_rule.as_slice());
                let exists = match cmd.status() {
                    Ok(ExitStatus(0)) => true,
                    Ok(ExitStatus(1)) => false,
                    val => return Err(
                        format!("Error running command {}: {}", cmd, val)),
                };
                debug!("Checked {} -> {}", check_rule, exists);

                if exists {
                    info!("Rule {} already setup. Skipping...", rule);
                } else {
                    let mut cmd = iptables.clone();
                    cmd.args(rule.as_slice());
                    debug!("Running {}", rule);
                    match cmd.status() {
                        Ok(ExitStatus(0)) => {},
                        val => return Err(
                            format!("Error setting up iptables {}: {}",
                                cmd, val)),
                    }
                }
            }
        }
    }

    Ok(0)
}

pub fn destroy_netns(_config: &Config, mut args: Vec<String>)
    -> Result<int, String>
{
    let interface_name = "vagga".to_string();
    let network = "172.18.255.0/30".to_string();
    let host_ip = "172.18.255.1".to_string();
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
        ap.refer(&mut iptables)
            .add_option(&["--no-iptables"], box StoreFalse,
                "Do not remove iptables rules (useful you have firewall \
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
    let runtime_dir = namespace_dir();
    let netns_file = runtime_dir.join("netns");
    let userns_file = runtime_dir.join("userns");

    let mut commands = vec!();

    // If we are root we may skip sudo
    let mut umount_cmd = Command::new("sudo");
    umount_cmd.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2));
    umount_cmd.arg("umount");

    let mut iptcmd = Command::new("sudo");
    iptcmd.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2));
    iptcmd.arg("iptables");

    let mut cmd = umount_cmd.clone();
    cmd.arg(&netns_file);
    commands.push(cmd);

    let mut cmd = umount_cmd.clone();
    cmd.arg(&userns_file);
    commands.push(cmd);

    if iptables {
        let mut cmd = iptcmd.clone();
        cmd.args(["-t", "nat", "-D", "POSTROUTING",
                  "-s", network.as_slice(), "-j", "MASQUERADE"]);
        commands.push(cmd);

        let mut cmd = iptcmd.clone();
        cmd.args(["-D", "INPUT",
                  "-i", interface_name.as_slice(),
                  "-d", "127.0.0.1",
                  "-j", "ACCEPT"]);
        commands.push(cmd);

        let mut cmd = iptcmd.clone();
        cmd.args(["-t", "nat", "-D", "PREROUTING",
                  "-p", "tcp", "-i", "vagga",
                  "-d", host_ip.as_slice(), "--dport", "53",
                  "-j", "DNAT", "--to-destination", "127.0.0.1"]);
        commands.push(cmd);

        let mut cmd = iptcmd.clone();
        cmd.args(["-t", "nat", "-D", "PREROUTING",
                  "-p", "udp", "-i", "vagga",
                  "-d", host_ip.as_slice(), "--dport", "53",
                  "-j", "DNAT", "--to-destination", "127.0.0.1"]);
        commands.push(cmd);
    }

    println!("We will run network setup commands with sudo.");
    println!("You may need to enter your password.");
    println!("");
    println!("The following commands will be run:");
    for cmd in commands.iter() {
        println!("    {}", cmd);
    }

    if !dry_run {
        for cmd in commands.iter() {
            match cmd.status() {
                Ok(ExitStatus(0)) => {}
                val => {
                    error!("Error running command {}: {}", cmd, val);
                }
            }
        }
        if let Err(e) = unlink(&netns_file) {
            error!("Error removing file: {}", e);
        }
        if let Err(e) = unlink(&userns_file) {
            error!("Error removing file: {}", e);
        }
    }


    Ok(0)
}

pub fn is_netns_set_up() -> bool {
    let nsdir = namespace_dir();
    return nsdir.join("userns").exists() && nsdir.join("netns").exists();
}

pub fn join_netns() -> Result<(), String> {
    let nsdir = namespace_dir();
    try!(set_namespace(&nsdir.join("userns"), NewUser)
        .map_err(|e| format!("Error setting userns: {}", e)));
    try!(set_namespace(&nsdir.join("netns"), NewNet)
        .map_err(|e| format!("Error setting networkns: {}", e)));
    Ok(())
}

pub fn run_in_netns(workdir: &Path, cname: String, args: Vec<String>)
    -> Result<int, String>
{
    try!(join_netns());
    user::run_wrapper(workdir, cname, args, false)
}

pub fn get_nameservers() -> Result<Vec<String>, String> {
    File::open(&Path::new("/etc/resolv.conf"))
        .map(BufferedReader::new)
        .and_then(|mut f| {
            let mut ns = vec!();
            for line in f.lines() {
                let line = try!(line);
                if line.as_slice().starts_with("nameserver ") {
                    ns.push(line[11..].trim().to_string());
                }
            }
            Ok(ns)
        })
        .map_err(|e| format!("Can't read resolf.conf: {}", e))
}
