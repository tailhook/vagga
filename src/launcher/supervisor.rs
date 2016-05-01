use std::collections::BTreeSet;
use std::collections::{HashMap};
use std::io::{self, stdout, stderr, Write};
use std::path::Path;

use time::{SteadyTime, Duration};
use libmount::Tmpfs;
use argparse::{ArgumentParser, List};
use signal::trap::Trap;
use nix::sys::signal::{SIGINT, SIGTERM, SIGCHLD, SIGTTIN, SIGTTOU};
use nix::sys::signal::{SIGQUIT, SIGKILL};
use unshare::{Command, Namespace, reap_zombies};

use options::build_mode::{build_mode, BuildMode};
use container::nsutil::{set_namespace, unshare_namespace};
use config::{Settings};
use config::command::{SuperviseInfo, Networking};
use config::command::SuperviseMode::{stop_on_failure};
use config::command::ChildCommand::{BridgeCommand};
use tty_util::{TtyGuard};
use super::network;
use super::build::{build_container};
use file_util::create_dir;
use process_util::{convert_status, killpg};
use super::wrap::Wrapper;


pub fn run_supervise_command(settings: &Settings, workdir: &Path,
    sup: &SuperviseInfo, cmdname: String, mut args: Vec<String>,
    mut bmode: BuildMode)
    -> Result<i32, String>
{
    let mut only: Vec<String> = Vec::new();
    let mut exclude: Vec<String> = Vec::new();
    if sup.mode != stop_on_failure {
        panic!("Only stop-on-failure mode implemented");
    }
    {
        args.insert(0, "vagga ".to_string() + &cmdname);
        let mut ap = ArgumentParser::new();
        ap.set_description(sup.description.as_ref().map(|x| &x[..])
            .unwrap_or("Run multiple processes simultaneously"));
        ap.refer(&mut only).metavar("PROCESS_NAME_OR_TAG")
            .add_option(&["--only"], List, "
                Only run specified processes.
                This matches both names and tags");
        ap.refer(&mut exclude).metavar("PROCESS_NAME_OR_TAG")
            .add_option(&["--exclude"], List, "
                Don't run specified processes.
                This excludes both names and tags");
        build_mode(&mut ap, &mut bmode);
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => {
                return Ok(122);
            }
        }
    }

    let mut containers = BTreeSet::new();
    let mut containers_in_netns = vec!();
    let mut bridges = vec!();
    let mut containers_host_net = vec!();
    let mut forwards = vec!();
    let mut ports = vec!();
    let mut versions = HashMap::new();
    let filtered_children = sup.children
        .iter().filter(|&(ref name, ref child)| {
            if only.len() > 0 {
                only.iter().find(|x| {
                    name == x || child.get_tags().iter().any(|t| t == *x)
                }).is_some()
            } else {
                exclude.iter().find(|x| {
                    name == x || child.get_tags().iter().any(|t| t == *x)
                }).is_none()
            }
        });
    for (name, child) in filtered_children {
        let cont = child.get_container();
        if !containers.contains(cont) {
            containers.insert(cont.to_string());
            let ver = try!(build_container(settings, cont, bmode));
            versions.insert(cont.to_string(), ver);
        }
        if let &BridgeCommand(_) = child {
            bridges.push(name.to_string());
        } else {
            if let Some(ref netw) = child.network() {
                containers_in_netns.push(name.to_string());
                for (ext_port, int_port) in netw.ports.iter() {
                     forwards.push((*ext_port, netw.ip.clone(), *int_port));
                    ports.push(*ext_port);
                }
            } else {
                containers_host_net.push(name.to_string());
            }
        }
    }
    containers_in_netns.extend(bridges.into_iter()); // Bridges are just last
    if containers_in_netns.len() > 0 && !network::is_netns_set_up() {
        return Err(format!("Network namespace is not set up. You need to run \
            vagga _create_netns first"));
    }
    debug!("Containers {} with host neworking, {} in netns",
        containers_host_net.len(), containers_in_netns.len());

    // Trap must be installed before tty_guard because TTY guard relies on
    // SIGTTOU and SIGTTIN be masked out
    let mut trap = Trap::trap(&[SIGINT, SIGQUIT,
                                SIGTERM, SIGCHLD, SIGTTOU, SIGTTIN]);
    let mut tty_guard = try!(TtyGuard::capture_tty()
        .map_err(|e| format!("Error handling tty: {}", e)));

    let mut children = HashMap::new();
    let mut error = false;
    for name in containers_host_net.iter() {
        let mut cmd: Command = Wrapper::new(
            Some(&versions[sup.children[name].get_container()]),
            settings);
        cmd.workdir(workdir);
        cmd.userns();
        cmd.arg(&cmdname);
        cmd.arg(&name);
        cmd.make_group_leader(true);
        match cmd.spawn() {
            Ok(child) => { children.insert(child.pid(), (name, child)); }
            Err(e) => {
                if !error {
                    println!(
                        "---------- \
                        Process {} could not be run: {}. Shutting down \
                        -----------",
                        name, e);
                    error = true;
                }
            }
        }
    }
    let port_forward_guard;
    if containers_in_netns.len() > 0 {
        let gwdir = network::namespace_dir();
        let nsdir = gwdir.join("children");
        if !nsdir.exists() {
            try_msg!(create_dir(&nsdir, false),
                     "Failed to create dir: {err}");
        }
        try!(network::join_gateway_namespaces());
        try!(unshare_namespace(Namespace::Mount)
            .map_err(|e| format!("Failed to create mount namespace: {}", e)));
        try!(Tmpfs::new(&nsdir)
            .size_bytes(10 << 20)
            .mode(0o755)
            .mount().map_err(|e| format!("{}", e)));

        let bridge_ns = nsdir.join("bridge");
        let ip = try!(network::setup_bridge(&bridge_ns, &forwards));

        port_forward_guard = network::PortForwardGuard::new(
            &gwdir.join("netns"), ip, ports);
        try!(port_forward_guard.start_forwarding());

        for name in containers_in_netns.iter() {
            let child = sup.children.get(name).unwrap();
            let mut cmd: Command = Wrapper::new(
                Some(&versions[sup.children[name].get_container()]),
                settings);
            cmd.workdir(workdir);
            cmd.arg(&cmdname);
            cmd.arg(&name);

            try!(set_namespace(&bridge_ns, Namespace::Net)
                .map_err(|e| format!("Error setting netns: {}", e)));
            if let &BridgeCommand(_) = child {
                // Already setup by set_namespace
                // But also need to mount namespace_dir into container
                cmd.env("VAGGA_NAMESPACE_DIR", &nsdir);
            } else {
                let netw = child.network().unwrap();
                let net_ns;
                let uts_ns;
                net_ns = nsdir.join("net.".to_string() + &netw.ip);
                uts_ns = nsdir.join("uts.".to_string() + &netw.ip);
                // TODO(tailhook) support multiple commands with same IP
                try!(network::setup_container(&net_ns, &uts_ns,
                    &name, &netw.ip,
                    &netw.hostname.as_ref().unwrap_or(name)));
                try!(set_namespace(&net_ns, Namespace::Net)
                    .map_err(|e| format!("Error setting netns: {}", e)));
                try!(set_namespace(&uts_ns, Namespace::Uts)
                    .map_err(|e| format!("Error setting netns: {}", e)));
            }

            cmd.make_group_leader(true);
            match cmd.spawn() {
                Ok(child) => { children.insert(child.pid(), (name, child)); }
                Err(e) => {
                    if !error {
                        println!(
                            "---------- \
                            Process {} could not be run: {}. Shutting down \
                            -----------",
                            name, e);
                        error = true;
                    }
                }
            }
        }

        // Need to set network namespace back to bridge, to keep namespace
        // alive. Otherwise bridge is dropped, and no connectivity between
        // containers.
        try!(set_namespace(&bridge_ns, Namespace::Net)
            .map_err(|e| format!("Error setting netns: {}", e)));
    }

    let mut errcode = 0;
    if error {
        errcode = 127;
        for &(_, ref child) in children.values() {
            child.signal(SIGTERM).ok();
        }
    } else {
        // Normal loop
        assert!(children.len() > 0);
        'signal_loop: for signal in trap.by_ref() {
            match signal {
                SIGINT => {
                    // SIGINT is usually a Ctrl+C, if we trap it here
                    // child process hasn't controlling terminal,
                    // so we send the signal to the child process
                    writeln!(&mut stderr(), "Received SIGINT signal. \
                        Waiting the processes to stop...").ok();
                    for &(cmd, ref child) in children.values() {
                        if unsafe { killpg(child.pid(), SIGTERM) } < 0 {
                             error!("Error sending SIGTERM to {:?}: {}", cmd,
                                io::Error::last_os_error());
                        }
                    }
                    errcode = 128+SIGINT;
                    break;
                }
                SIGTERM|SIGQUIT => {
                    // SIGTERM is usually sent to a specific process so we
                    // forward it to children
                    writeln!(&mut stderr(), "Received {} signal, \
                        propagating", signal).ok();
                    for &(_, ref child) in children.values() {
                        child.signal(SIGTERM).ok();
                    }
                    errcode = 128+signal;
                    break;
                }
                SIGCHLD => {
                    for (pid, status) in reap_zombies() {
                        if let Some((name, _)) = children.remove(&pid) {
                            errcode = convert_status(status);
                            println!(
                                "---------- \
                                Process {}:{} {}. Shutting down \
                                -----------",
                                name, pid, status);
                            for (pid, _) in reap_zombies() {
                                children.remove(&pid);
                            }
                            for &(_, ref child) in children.values() {
                                child.signal(SIGTERM).ok();
                            }
                            break 'signal_loop;
                        }
                    }
                    try!(tty_guard.check().map_err(|e|
                        format!("Error handling tty: {}", e)));
                    if children.len() == 0 {
                        break;
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    // Stopping loop
    if children.len() > 0 {
        let timeo = sup.kill_unresponsive_after;
        let mut deadline = SteadyTime::now() + Duration::seconds(timeo as i64);
        loop {
            match trap.wait(deadline) {
                Some(SIGINT) => {}
                Some(SIGTERM) => {
                    for &(_, ref child) in children.values() {
                        child.signal(SIGTERM).ok();
                    }
                }
                Some(SIGCHLD) => {
                    for (pid, _) in reap_zombies() {
                        children.remove(&pid);
                    }
                    try!(tty_guard.check().map_err(|e|
                        format!("Error handling tty: {}", e)));
                    if children.len() == 0 {
                        break;
                    }
                }
                Some(_) => unreachable!(),
                None => {
                    println!(
                        "---------- \
                        Processes {:?} are still alive. Killing ... \
                        -----------\n\
                        To prevent killing of the processes set \
                        ``kill-unresponsive-after`` to some larger value",
                        children.values().map(|&(name, _)| name)
                            .collect::<Vec<_>>());
                    for &(_, ref child) in children.values() {
                        child.signal(SIGKILL).ok();
                    }
                    // Basically this deadline should never happen
                    deadline = SteadyTime::now() + Duration::seconds(3600);
                }
            }
        }
    }

    if errcode == 0 {
        if let Some(ref epilog) = sup.epilog {
            print!("{}", epilog);
        }
    }

    Ok(errcode)
}
