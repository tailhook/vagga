use std::cell::Cell;
use std::collections::BTreeSet;
use std::env::current_exe;
use std::io::{stdout, stderr};
use std::path::Path;
use std::fs::PathExt;
use std::rc::Rc;

use argparse::{ArgumentParser};

use container::mount::{mount_tmpfs};
use container::nsutil::{set_namespace, unshare_namespace};
use container::monitor::{Monitor, Executor};
use container::monitor::MonitorResult::{Exit, Killed};
use container::monitor::MonitorStatus;
use container::container::{Command};
use container::container::Namespace::{NewNet, NewUts, NewMount};
use config::Config;
use config::command::{SuperviseInfo, Networking};
use config::command::SuperviseMode::{stop_on_failure};
use config::command::ChildCommand::{BridgeCommand};

use super::network;
use super::user::{run_wrapper, common_child_command_env};
use super::build::build_container;
use super::super::file_util::create_dir;


pub struct RunChild<'a> {
    name: Rc<String>,
    command: Option<Command>,
    running: &'a Cell<bool>
}

impl<'a> Executor for RunChild<'a> {
    fn command(&mut self) -> Command {
        return self.command.take().expect("Command can't be run twice");
    }
    fn finish(&mut self, status: i32) -> MonitorStatus {
        if self.running.get() {
            println!(
                "---------- \
                Process {} exited with code {}. Shutting down \
                -----------",
                self.name, status);
            self.running.set(false);
        }
        MonitorStatus::Shutdown(status)
    }
}


pub fn run_supervise_command(config: &Config, workdir: &Path,
    sup: &SuperviseInfo, cmdname: String, mut args: Vec<String>)
    -> Result<i32, String>
{
    if sup.mode != stop_on_failure {
        panic!("Only stop-on-failure mode implemented");
    }
    {
        args.insert(0, "vagga ".to_string() + &cmdname);
        let mut ap = ArgumentParser::new();
        ap.set_description(sup.description.as_ref().map(|x| &x[..])
            .unwrap_or("Run multiple processes simultaneously"));
        // TODO(tailhook) implement --only and --exclude
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
    for (name, child) in sup.children.iter() {
        let cont = child.get_container();
        if !containers.contains(cont) {
            containers.insert(cont.to_string());
            try!(build_container(config, cont));
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
    let running = Cell::new(true);
    let mut mon = Monitor::new();
    for name in containers_host_net.iter() {
        let mut cmd = Command::new("wrapper".to_string(),
            &current_exe().unwrap().parent().unwrap()
            .join("vagga_wrapper"));
        cmd.keep_sigmask();
        cmd.arg(&cmdname);
        cmd.arg(&name);
        common_child_command_env(&mut cmd, Some(workdir));
        cmd.container();
        cmd.set_max_uidmap();
        let name = Rc::new(name.clone());
        mon.add(name.clone(), Box::new(RunChild {
            name: name,
            command: Some(cmd),
            running: &running,
        }));
    }
    let mut port_forward_guard;
    if containers_in_netns.len() > 0 {
        let gwdir = network::namespace_dir();
        let nsdir = gwdir.join("children");
        if !nsdir.exists() {
            try_msg!(create_dir(&nsdir, false),
                     "Failed to create dir: {err}");
        }
        try!(network::join_gateway_namespaces());
        try!(unshare_namespace(NewMount)
            .map_err(|e| format!("Failed to create mount namespace: {}", e)));
        try!(mount_tmpfs(&nsdir, "size=10m"));

        let bridge_ns = nsdir.join("bridge");
        let ip = try!(network::setup_bridge(&bridge_ns, &forwards));

        port_forward_guard = network::PortForwardGuard::new(
            &gwdir.join("netns"), ip, ports);
        try!(port_forward_guard.start_forwarding());

        for name in containers_in_netns.iter() {
            let child = sup.children.get(name).unwrap();
            let mut cmd = Command::new("wrapper".to_string(),
                &current_exe().unwrap().parent().unwrap()
                .join("vagga_wrapper"));
            cmd.keep_sigmask();
            cmd.arg(&cmdname);
            cmd.arg(&name);
            common_child_command_env(&mut cmd, Some(workdir));
            cmd.container();

            try!(set_namespace(&bridge_ns, NewNet)
                .map_err(|e| format!("Error setting netns: {}", e)));
            if let &BridgeCommand(_) = child {
                // Already setup by set_namespace
                // But also need to mount namespace_dir into container
                cmd.set_env("VAGGA_NAMESPACE_DIR".to_string(),
                            nsdir.display().to_string());
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
                try!(set_namespace(&net_ns, NewNet)
                    .map_err(|e| format!("Error setting netns: {}", e)));
                try!(set_namespace(&uts_ns, NewUts)
                    .map_err(|e| format!("Error setting netns: {}", e)));
            }

            let name = Rc::new(name.clone());
            mon.add(name.clone(), Box::new(RunChild {
                name: name.clone(),
                command: Some(cmd),
                running: &running,
            }));
            try!(mon.force_start(name));  // ensure run in correct sequence
        }

        // Need to set network namespace back to bridge, to keep namespace
        // alive. Otherwise bridge is dropped, and no connectivity between
        // containers.
        try!(set_namespace(&bridge_ns, NewNet)
            .map_err(|e| format!("Error setting netns: {}", e)));
    }
    match mon.run() {
        Killed => Ok(143),
        Exit(val) => Ok(val),
    }
}
