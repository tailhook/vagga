use std::env;
use std::fs::{remove_file};
use std::fs::{File};
use std::io::{stdout, stderr, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::collections::HashSet;
use std::os::unix::io::AsRawFd;

use argparse::{ArgumentParser};
use argparse::{StoreTrue, StoreFalse};
use digest_traits::Digest;
use libc::{geteuid};
use libmount::BindMount;
use log::Level::Debug;
use rand::{thread_rng, Rng};
use serde_json;
use sha2::Sha256;
use unshare::{Command, Stdio, Fd, Namespace};

use crate::config::Config;
use crate::container::uidmap::get_max_uidmap;
use crate::container::network::detect_local_dns;
use crate::container::nsutil::{set_namespace};
use crate::digest::hex;
use crate::file_util::Dir;
use crate::process_util::{set_uidmap, env_command, run_success, cmd_err, cmd_show};

static MAX_INTERFACES: u32 = 2048;

pub struct PortForwardGuard {
    nspath: PathBuf,
    ip: String,
    ports: Vec<u16>,
}

pub fn namespace_dir() -> PathBuf {
    let uid = unsafe { geteuid() };
    env::var("XDG_RUNTIME_DIR")
        .map(|v| Path::new(&v).join("vagga"))
        .unwrap_or(PathBuf::from(format!("/tmp/vagga-{}", uid)))
}

pub fn sudo_ip_cmd() -> Command {
    // If we are root we may skip sudo
    let mut ip_cmd = env_command("sudo");
    ip_cmd.stdin(Stdio::null());
    ip_cmd.arg("ip");
    ip_cmd
}

pub fn sudo_sysctl() -> Command {
    // If we are root we may skip sudo
    let mut sysctl = env_command("sudo");
    sysctl.stdin(Stdio::null());
    sysctl.arg("sysctl");
    sysctl
}

pub fn ip_cmd() -> Command {
    let mut ip_cmd = env_command("ip");
    ip_cmd.stdin(Stdio::null());
    ip_cmd
}

pub fn sudo_mount() -> Command {
    // If we are root we may skip sudo
    let mut mount_cmd = env_command("sudo");
    mount_cmd.stdin(Stdio::null());
    mount_cmd.arg("mount");
    mount_cmd
}

pub fn sudo_umount() -> Command {
    // If we are root we may skip sudo
    let mut umount_cmd = env_command("sudo");
    umount_cmd.stdin(Stdio::null());
    umount_cmd.arg("umount");
    umount_cmd
}

pub fn sudo_iptables() -> Command {
    // If we are root we may skip sudo
    let mut iptables_cmd = env_command("sudo");
    iptables_cmd.stdin(Stdio::null());
    iptables_cmd.arg("iptables");
    iptables_cmd
}

pub fn iptables() -> Command {
    // If we are root we may skip sudo
    let mut iptables_cmd = env_command("iptables");
    iptables_cmd.stdin(Stdio::null());
    iptables_cmd
}

pub fn busybox() -> Command {
    let mut busybox = Command::new(
        &env::current_exe().unwrap().parent().unwrap()
        .join("busybox"));
    busybox.stdin(Stdio::null());
    busybox
}


pub fn create_netns(_config: &Config, mut args: Vec<String>)
    -> Result<i32, String>
{
    let interface_name = "vagga".to_string();
    let network = "172.23.255.0/30".to_string();
    let host_ip_net = "172.23.255.1/30".to_string();
    let host_ip = "172.23.255.1".to_string();
    let guest_ip = "172.23.255.2/30".to_string();
    let mut dry_run = false;
    let mut iptables = true;
    {
        args.insert(0, "vagga _create_netns".to_string());
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Set's up network namespace for subsequent container runs
            ");
        ap.refer(&mut dry_run)
            .add_option(&["--dry-run"], StoreTrue,
                "Do not run commands, only show");
        ap.refer(&mut iptables)
            .add_option(&["--no-iptables"], StoreFalse,
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
        try_msg!(Dir::new(&runtime_dir).mode(0o755).create(),
            "Can't create runtime_dir: {err}");
    }

    let netns_file = runtime_dir.join("netns");
    let userns_file = runtime_dir.join("userns");

    if netns_file.exists() || userns_file.exists() {
        return Err("Namespaces already created".to_string());
    }

    let mut commands = vec!();

    let mut nforward = String::with_capacity(100);
    File::open(&Path::new("/proc/sys/net/ipv4/ip_forward"))
        .and_then(|mut f| f.read_to_string(&mut nforward))
        .map_err(|e| format!("Can't read sysctl: {}", e))?;

    if nforward[..].trim() == "0" {
        let mut cmd = sudo_sysctl();
        cmd.arg("net.ipv4.ip_forward=1");
        commands.push(cmd);
    } else {
        info!("Sysctl is ok [{}]", nforward[..].trim());
    }

    if !dry_run && !commands.is_empty() {
        println!("");
        println!("The following commands will be run first:");
        // need to setup ip_forward before creating new namespaces
        for cmd in commands.drain(..) {
            run_success(cmd)?;
        }
    }

    let mut cmd = Command::new(env::current_exe().unwrap());
    cmd.arg("__setup_netns__");
    cmd.unshare(&[Namespace::Net]);
    set_uidmap(&mut cmd, &get_max_uidmap().unwrap(), true);
    cmd.env_clear();
    // we never need proxy env vars here
    cmd.env("TERM".to_string(),
            env::var_os("TERM").unwrap_or(From::from("dumb")));
    if let Ok(x) = env::var("PATH") {
        cmd.env("PATH".to_string(), x);
    }
    if let Ok(x) = env::var("RUST_LOG") {
        cmd.env("RUST_LOG".to_string(), x);
    }
    if let Ok(x) = env::var("RUST_BACKTRACE") {
        cmd.env("RUST_BACKTRACE".to_string(), x);
    }
    if let Ok(x) = env::var("VAGGA_DEBUG_CMDENV") {
        cmd.env("VAGGA_DEBUG_CMDENV", x);
    }
    cmd.arg("gateway");
    cmd.arg("--guest-ip");
    cmd.arg(&guest_ip);
    cmd.arg("--gateway-ip");
    cmd.arg(&host_ip);
    cmd.arg("--network");
    cmd.arg(&network);
    cmd.file_descriptor(3, Fd::piped_read());
    let child = if dry_run { None } else {
        Some(cmd.spawn().map_err(|e| cmd_err(&cmd, e))?)
    };
    let child_pid = child.as_ref().map(|x| x.pid()).unwrap_or(123456);

    println!("We will run network setup commands with sudo.");
    println!("You may need to enter your password.");

    let mut cmd = sudo_ip_cmd();
    cmd.args(&["link", "add", "vagga_guest", "type", "veth",
              "peer", "name", &interface_name[..]]);
    commands.push(cmd);

    let mut cmd = sudo_ip_cmd();
    cmd.args(&["link", "set", "vagga_guest", "netns"]);
    cmd.arg(&format!("{}", child_pid));
    commands.push(cmd);

    let mut cmd = sudo_ip_cmd();
    cmd.args(&["addr", "add", &host_ip_net[..],
              "dev", &interface_name[..]]);
    commands.push(cmd);

    let mut cmd = sudo_ip_cmd();
    cmd.args(&["link", "set", &interface_name, "up"]);
    commands.push(cmd);

    let mut cmd = sudo_sysctl();
    cmd.arg("net.ipv4.conf.vagga.route_localnet=1");
    commands.push(cmd);

    let local_dns = detect_local_dns().map_err(|e| format!("{}", e))?;

    if !dry_run {
        File::create(&netns_file)
            .map_err(|e| format!("Error creating netns file: {}", e))?;
        File::create(&userns_file)
            .map_err(|e| format!("Error creating userns file: {}", e))?;
    }

    let mut cmd = sudo_mount();
    cmd.arg("--bind");
    cmd.arg(format!("/proc/{}/ns/net", child_pid));
    cmd.arg(netns_file);
    commands.push(cmd);

    let mut cmd = sudo_mount();
    cmd.arg("--bind");
    cmd.arg(format!("/proc/{}/ns/user", child_pid));
    cmd.arg(userns_file);
    commands.push(cmd);

    let mut iprules = vec!();
    if let Some(ref ip) = local_dns {
        iprules.push(vec!("-I", "INPUT",
                          "-i", &interface_name[..],
                          "-d", ip,
                          "-j", "ACCEPT"));
        //  The "tcp" rule doesn't actually work for now for dnsmasq
        //  because dnsmasq tries to find out real source IP.
        //  It may work for bind though.
        iprules.push(vec!("-t", "nat", "-I", "PREROUTING",
                          "-p", "tcp", "-i", "vagga",
                          "-d", &host_ip[..], "--dport", "53",
                          "-j", "DNAT", "--to-destination", ip));
        iprules.push(vec!("-t", "nat", "-I", "PREROUTING",
                          "-p", "udp", "-i", "vagga",
                          "-d", &host_ip[..], "--dport", "53",
                          "-j", "DNAT", "--to-destination", ip));
    }
    iprules.push(vec!("-t", "nat", "-A", "POSTROUTING",
                      "-s", &network[..], "-j", "MASQUERADE"));


    println!("");
    println!("The following commands will be run:");
    for cmd in commands.iter() {
        println!("    {}", cmd_show(&cmd));
    }


    if iptables {
        println!("");
        println!("The following iptables rules will be established:");

        for rule in iprules.iter() {
            print!("    iptables");
            for i in rule.iter() {
                print!(" {}", i);
            }
            println!("");
        }
    }

    if !dry_run {
        for cmd in commands.into_iter() {
            run_success(cmd)?;
        }

        let mut child = child.unwrap();
        child.take_pipe_writer(3).unwrap().write_all(b"ok")
            .map_err(|e| format!("Error writing to pipe: {}", e))?;

        match child.wait() {
            Ok(status) if status.success() => {}
            Ok(status) => return Err(
                format!("vagga_setup_netns {}", status)),
            Err(e) => return Err(format!("child wait error: {}", e)),
        }
        if iptables {
            for rule in iprules.iter() {
                let mut cmd = sudo_iptables();

                // Message not an error actually
                // Let's print it only on debug log level
                if !log_enabled!(Debug) {
                    cmd.stderr(Stdio::null());
                }

                let mut check_rule = rule.clone();
                for item in check_rule.iter_mut() {
                    if *item == "-A" || *item == "-I" {
                        *item = "-C";
                    }
                }
                cmd.args(&check_rule[..]);
                debug!("Running {}", cmd_show(&cmd));
                let exists = match cmd.status() {
                    Ok(status) if status.success() => true,
                    Ok(status) if status.code() == Some(1) => false,
                    Ok(status) => return Err(cmd_err(&cmd, status)),
                    Err(err) => return Err(cmd_err(&cmd, err)),
                };
                debug!("Checked {:?} -> {}", check_rule, exists);

                if exists {
                    info!("Rule {:?} already setup. Skipping...", rule);
                } else {
                    let mut cmd = sudo_iptables();
                    cmd.args(&rule[..]);
                    run_success(cmd)?;
                }
            }
        }
    }

    Ok(0)
}

pub fn destroy_netns(_config: &Config, mut args: Vec<String>)
    -> Result<i32, String>
{
    let interface_name = "vagga".to_string();
    let network = "172.23.255.0/30".to_string();
    let host_ip = "172.23.255.1".to_string();
    let mut dry_run = false;
    let mut iptables = true;
    {
        args.insert(0, "vagga _create_netns".to_string());
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Set's up network namespace for subsequent container runs
            ");
        ap.refer(&mut dry_run)
            .add_option(&["--dry-run"], StoreTrue,
                "Do not run commands, only show");
        ap.refer(&mut iptables)
            .add_option(&["--no-iptables"], StoreFalse,
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

    let mut cmd = sudo_umount();
    cmd.arg(&netns_file);
    commands.push(cmd);

    let mut cmd = sudo_umount();
    cmd.arg(&userns_file);
    commands.push(cmd);

    if iptables {
        let mut cmd = sudo_iptables();
        cmd.args(&["-t", "nat", "-D", "POSTROUTING",
                   "-s", &network[..], "-j", "MASQUERADE"]);
        commands.push(cmd);

        let local_dns = detect_local_dns().map_err(|e| format!("{}", e))?;

        if let Some(ref ip) = local_dns {
            let mut cmd = sudo_iptables();
            cmd.args(&["-D", "INPUT",
                       "-i", &interface_name[..],
                       "-d", ip,
                       "-j", "ACCEPT"]);
            commands.push(cmd);

            let mut cmd = sudo_iptables();
            cmd.args(&["-t", "nat", "-D", "PREROUTING",
                       "-p", "tcp", "-i", "vagga",
                       "-d", &host_ip[..], "--dport", "53",
                       "-j", "DNAT", "--to-destination", ip]);
            commands.push(cmd);

            let mut cmd = sudo_iptables();
            cmd.args(&["-t", "nat", "-D", "PREROUTING",
                       "-p", "udp", "-i", "vagga",
                       "-d", &host_ip[..], "--dport", "53",
                       "-j", "DNAT", "--to-destination", ip]);
            commands.push(cmd);
        }
    }

    println!("We will run network setup commands with sudo.");
    println!("You may need to enter your password.");
    println!("");
    println!("The following commands will be run:");
    for cmd in commands.iter() {
        println!("    {}", cmd_show(&cmd));
    }

    if !dry_run {
        for cmd in commands.into_iter() {
            run_success(cmd).map_err(|e| error!("{}", e)).ok();
        }
        if let Err(e) = remove_file(&netns_file) {
            error!("Error removing file: {}", e);
        }
        if let Err(e) = remove_file(&userns_file) {
            error!("Error removing file: {}", e);
        }
    }


    Ok(0)
}

pub fn is_netns_set_up() -> bool {
    let nsdir = namespace_dir();
    return nsdir.join("userns").exists() && nsdir.join("netns").exists();
}

pub fn join_gateway_namespaces() -> Result<(), String> {
    let nsdir = namespace_dir();
    set_namespace(nsdir.join("userns"), Namespace::User)
        .map_err(|e| format!("Error setting userns: {}", e))?;
    set_namespace(nsdir.join("netns"), Namespace::Net)
        .map_err(|e| format!("Error setting networkns: {}", e))?;
    Ok(())
}

fn get_interfaces() -> Result<HashSet<u32>, String> {
    File::open(&Path::new("/proc/net/dev"))
        .map(BufReader::new)
        .and_then(|f| {
            let mut lineiter = f.lines();
            let mut result = HashSet::with_capacity(MAX_INTERFACES as usize);
            lineiter.next().unwrap()?;  // Two header lines
            lineiter.next().unwrap()?;
            for line in lineiter {
                let line = line?;
                let line = line.trim();
                let end = line.find(':');
                if line.starts_with("ch") && end.is_some() {
                    if let Ok(num) = FromStr::from_str(
                        &line[3..end.unwrap()])
                    {
                        result.insert(num);
                    }
                }
            }
            return Ok(result);
        })
        .map_err(|e| format!("Can't read interfaces: {}", e))
}

fn get_unused_inteface_no() -> Result<u32, String> {
    // Algorithm is not perfect but should be good enough as there are 2048
    // slots in total, and average user only runs a couple of commands
    // simultaneously. It fails miserably only if there are > 100 or they
    // are spawning too often.
    let busy = get_interfaces()?;
    let start = thread_rng().gen_range(0u32, MAX_INTERFACES - 100);
    for index in start..MAX_INTERFACES {
        if busy.contains(&index) {
            continue;
        }
        return Ok(index);
    }
    return Err(format!("Can't find unused inteface"));
}

pub fn setup_bridge(link_to: &Path, port_forwards: &Vec<(u16, String, u16)>)
    -> Result<String, String>
{
    let index = get_unused_inteface_no()?;

    let eif = format!("ch{}", index);
    let iif = format!("ch{}c", index);
    let eip = format!("172.23.{}.{}", 192 + (index*4)/256, (index*4 + 1) % 256);
    let iip = format!("172.23.{}.{}", 192 + (index*4)/256, (index*4 + 2) % 256);

    File::create(link_to)
        .map_err(|e| format!("Can't create namespace file {:?}: {}",
                             link_to, e))?;


    let mut cmd = ip_cmd();
    cmd.args(&["link", "add", &eif[..], "type", "veth",
               "peer", "name", &iif[..]]);
    run_success(cmd)?;

    let mut cmd = ip_cmd();
    cmd.args(&["addr", "add"]);
    cmd.arg(eip.clone() + "/30").arg("dev").arg(&eif);
    run_success(cmd)?;

    let mut cmd = ip_cmd();
    cmd.args(&["link", "set", "dev", &eif[..], "up"]);
    run_success(cmd)?;

    let mut cmd = Command::new(env::current_exe().unwrap());
    cmd.arg("__setup_netns__");
    cmd.args(&["bridge",
        "--interface", &iif[..],
        "--ip", &iip[..],
        "--gateway-ip", &eip[..],
        "--port-forwards", &serde_json::to_string(port_forwards).unwrap()[..],
        ]);
    cmd.unshare(&[Namespace::Net]);
    cmd.env_clear();
    // we never need proxy env vars here
    cmd.env("TERM".to_string(),
            env::var_os("TERM").unwrap_or(From::from("dumb")));
    if let Ok(x) = env::var("PATH") {
        cmd.env("PATH".to_string(), x);
    }
    if let Ok(x) = env::var("RUST_LOG") {
        cmd.env("RUST_LOG".to_string(), x);
    }
    if let Ok(x) = env::var("RUST_BACKTRACE") {
        cmd.env("RUST_BACKTRACE".to_string(), x);
    }
    if let Ok(x) = env::var("VAGGA_DEBUG_CMDENV") {
        cmd.env("VAGGA_DEBUG_CMDENV", x);
    }
    cmd.file_descriptor(3, Fd::piped_read());
    let mut child = cmd.spawn()
            .map_err(|e| cmd_err(&cmd, e))?;

    let mut cmd = ip_cmd();
    cmd.args(&["link", "set", "dev", &iif[..],
               "netns", &format!("{}", child.pid())[..]]);

    let res = BindMount::new(format!("/proc/{}/ns/net", child.pid()), link_to)
        .mount().map_err(|e| e.to_string())
        .and(run_success(cmd));

    child.take_pipe_writer(3).unwrap().write_all(b"ok")
        .map_err(|e| format!("Error writing to pipe: {}", e))?;

    match child.wait() {
        Ok(status) if status.success() => {}
        Ok(status) => return Err(format!("vagga_setup_netns {}", status)),
        Err(e) => return Err(format!("wait child error: {}", e)),
    }
    match res {
        Ok(()) => Ok(iip),
        Err(e) => {
            let mut cmd = ip_cmd();
            cmd.args(&["link", "del", &eif[..]]);
            run_success(cmd)?;
            Err(e)
        }
    }
}

pub fn setup_container(link_net: &Path, link_uts: &Path, name: &str,
    ip: &str, hostname: &str)
    -> Result<(), String>
{
    let eif = if name.as_bytes().len() > 14 {
        let mut hash = Sha256::new();
        hash.update(name.as_bytes());
        format!("eh{:.12x}", hex(&hash))
    } else {
        name.to_string()
    };
    let iif = eif.clone() + "g";

    File::create(link_net)
        .map_err(|e| format!("Can't create namespace file {:?}: {}",
            link_net, e))?;
    File::create(link_uts)
        .map_err(|e| format!("Can't create namespace file {:?}: {}",
            link_uts, e))?;

    let mut cmd = ip_cmd();
    cmd.args(&["link", "add", &eif[..], "type", "veth",
               "peer", "name", &iif[..]]);
    run_success(cmd)?;

    let mut cmd = ip_cmd();
    cmd.args(&["link", "set", "dev", &eif[..], "up"]);
    run_success(cmd)?;

    let mut cmd = busybox();
    cmd.args(&["brctl", "addif", "children", &eif[..]]);
    run_success(cmd)?;

    let mut cmd = Command::new(env::current_exe().unwrap());
    cmd.arg("__setup_netns__");
    cmd.args(&["guest", "--interface", &iif[..],
                        "--ip", &ip[..],
                        "--hostname", hostname,
                        "--gateway-ip", "172.23.0.254"]);
    cmd.unshare(&[Namespace::Net, Namespace::Uts]);
    // we never need proxy env vars here
    cmd.env("TERM".to_string(),
            env::var_os("TERM").unwrap_or(From::from("dumb")));
    if let Ok(x) = env::var("PATH") {
        cmd.env("PATH".to_string(), x);
    }
    if let Ok(x) = env::var("RUST_LOG") {
        cmd.env("RUST_LOG".to_string(), x);
    }
    if let Ok(x) = env::var("RUST_BACKTRACE") {
        cmd.env("RUST_BACKTRACE".to_string(), x);
    }
    if let Ok(x) = env::var("VAGGA_DEBUG_CMDENV") {
        cmd.env("VAGGA_DEBUG_CMDENV", x);
    }
    cmd.file_descriptor(3, Fd::piped_read());
    let mut child = cmd.spawn()
            .map_err(|e| cmd_err(&cmd, e))?;
    let pid = child.pid();

    let mut cmd = ip_cmd();
    cmd.args(&["link", "set", "dev", &iif[..],
               "netns", &format!("{}", pid)[..]]);

    let res = BindMount::new(format!("/proc/{}/ns/net", pid), link_net).mount()
        .and(BindMount::new(format!("/proc/{}/ns/uts", pid), link_uts).mount())
        .map_err(|e| e.to_string())
        .and(run_success(cmd));

    child.take_pipe_writer(3).unwrap().write_all(b"ok")
        .map_err(|e| format!("Error writing to pipe: {}", e))?;
    match child.wait() {
        Ok(status) if status.success() => {}
        Ok(status) => return Err(format!("vagga_setup_netns {}", status)),
        Err(e) => return Err(format!("wait child error: {}", e)),
    }

    match res {
        Ok(()) => Ok(()),
        Err(e) => {
            let mut cmd = ip_cmd();
            cmd.args(&["link", "del", &eif[..]]);
            run_success(cmd)?;
            Err(e)
        }
    }
}

pub struct IsolatedNetwork {
    pub userns: File,
    pub netns: File,
}

pub fn create_isolated_network() -> Result<IsolatedNetwork, String> {
    let mut cmd = Command::new(env::current_exe().unwrap());
    cmd.arg("__setup_netns__");
    cmd.arg("isolated");
    cmd.unshare(&[Namespace::User, Namespace::Net]);
    let uid_map = get_max_uidmap()?;
    set_uidmap(&mut cmd, &uid_map, true);
    cmd.env_clear();
    cmd.file_descriptor(3, Fd::piped_read());
    let mut child = cmd.spawn().map_err(|e| cmd_err(&cmd, e))?;
    let child_pid = child.pid();

    let netns_file = try_msg!(
        File::open(PathBuf::from(format!("/proc/{}/ns/net", child_pid))),
        "Cannot open netns file: {err}");
    let userns_file = try_msg!(
        File::open(PathBuf::from(format!("/proc/{}/ns/user", child_pid))),
        "Cannot open userns file: {err}");

    child.take_pipe_writer(3).unwrap().write_all(b"ok")
        .map_err(|e| format!("Error writing to pipe: {}", e))?;

    match child.wait() {
        Ok(status) if status.success() => {}
        Ok(status) => return Err(cmd_err(&cmd, status)),
        Err(e) => return Err(cmd_err(&cmd, e)),
    }

    Ok(IsolatedNetwork{userns: userns_file, netns: netns_file})
}

#[cfg(feature="containers")]
pub fn isolate_network() -> Result<(), String> {
    use nix::sched::{setns, CloneFlags};

    let isolated_net = try_msg!(
        create_isolated_network(),
        "Cannot create network namespace: {err}");
    try_msg!(setns(isolated_net.userns.as_raw_fd(), CloneFlags::CLONE_NEWUSER),
        "Cannot set user namespace: {err}");
    try_msg!(setns(isolated_net.netns.as_raw_fd(), CloneFlags::CLONE_NEWNET),
        "Cannot set network namespace: {err}");
    Ok(())
}

#[cfg(not(feature="containers"))]
pub fn isolate_network() -> Result<(), String> {
    unimplemented!()
}

impl PortForwardGuard {
    pub fn new(ns: &Path, ip: String, ports: Vec<u16>) -> PortForwardGuard {
        return PortForwardGuard {
            nspath: ns.to_path_buf(),
            ip: ip,
            ports: ports,
        };
    }
    pub fn start_forwarding(&self) -> Result<(), String> {
        set_namespace(&self.nspath, Namespace::Net)
            .map_err(|e| format!("Error joining namespace: {}", e))?;

        for port in self.ports.iter() {
            let mut cmd = iptables();
            cmd.args(&["-t", "nat", "-I", "PREROUTING",
                       "-p", "tcp", "-m", "tcp",
                       "--dport", &format!("{}", port)[..],
                       "-j", "DNAT",
                       "--to-destination", &self.ip[..]]);
            run_success(cmd)?;
        }

        Ok(())
    }
}

impl Drop for PortForwardGuard {
    fn drop(&mut self) {
        if let Err(e) = set_namespace(&self.nspath, Namespace::Net) {
            error!("Can't set namespace {:?}: {}. \
                    Unable to clean firewall rules", self.nspath, e);
            return;
        }
        for port in self.ports.iter() {
            let mut cmd = iptables();
            cmd.args(&["-t", "nat", "-D", "PREROUTING",
                       "-p", "tcp", "-m", "tcp",
                       "--dport", &format!("{}", port)[..],
                       "-j", "DNAT",
                       "--to-destination", &self.ip[..]]);
            run_success(cmd)
            .unwrap_or_else(|e| error!("Error deleting firewall rule: {}", e));
        }
    }
}
