use std::rc::Rc;
use std::os::{getenv};
use std::io::ALL_PERMISSIONS;
use std::os::self_exe_path;
use std::io::stdio::{stdout, stderr};
use std::io::fs::{mkdir};
use std::io::fs::PathExtensions;
use std::collections::TreeSet;

use argparse::{ArgumentParser};

use container::mount::{mount_tmpfs};
use container::nsutil::{set_namespace, unshare_namespace};
use container::monitor::{Monitor, RunOnce, Exit, Killed};
use container::container::{Command};
use container::container::{NewNet, NewUts, NewMount};
use config::Config;
use config::command::{main};
use config::command::{CommandInfo, SuperviseInfo, Networking, stop_on_failure};
use config::command::child::{BridgeCommand};

use super::network;


pub fn run_user_command(config: &Config, workdir: &Path,
    cmd: String, args: Vec<String>)
    -> Result<int, String>
{
    match config.commands.find(&cmd) {
        None => Err(format!("Command {} not found. \
                    Run vagga without arguments to see the list.", cmd)),
        Some(&main::Command(ref info))
        => run_simple_command(info, workdir, cmd, args),
        Some(&main::Supervise(ref sup))
        => run_supervise_command(config, workdir, sup, cmd, args),
    }
}

fn _common(cmd: &mut Command, workdir: &Path) {
    cmd.set_env("TERM".to_string(),
                getenv("TERM").unwrap_or("dumb".to_string()));
    if let Some(x) = getenv("PATH") {
        cmd.set_env("HOST_PATH".to_string(), x);
    }
    if let Some(x) = getenv("RUST_LOG") {
        cmd.set_env("RUST_LOG".to_string(), x);
    }
    if let Some(x) = getenv("RUST_BACKTRACE") {
        cmd.set_env("RUST_BACKTRACE".to_string(), x);
    }
    if let Some(x) = getenv("HOME") {
        cmd.set_env("VAGGA_USER_HOME".to_string(), x);
    }
    cmd.set_env("PWD".to_string(), Path::new("/work")
        .join(workdir)
        .display().to_string());
}

pub fn run_simple_command(cfg: &CommandInfo,
    workdir: &Path, cmdname: String, args: Vec<String>)
    -> Result<int, String>
{
    if let Some(_) = cfg.network {
        return Err(format!(
            "Network is not supported for !Command use !Supervise"))
    }
    run_wrapper(workdir, cmdname, args, cfg.network.is_none())
}

// TODO(tailhook) run not only for simple commands
pub fn run_wrapper(workdir: &Path, cmdname: String, args: Vec<String>,
    userns: bool)
    -> Result<int, String>
{
    let mut cmd = Command::new("wrapper".to_string(),
        self_exe_path().unwrap().join("vagga_wrapper"));
    cmd.keep_sigmask();
    cmd.arg(cmdname.as_slice());
    cmd.args(args.as_slice());
    _common(&mut cmd, workdir);
    cmd.container();
    if userns {
        cmd.set_max_uidmap();
    }
    match Monitor::run_command(cmd) {
        Killed => Ok(143),
        Exit(val) => Ok(val),
    }
}

fn run_supervise_command(_config: &Config, workdir: &Path,
    sup: &SuperviseInfo, cmdname: String, mut args: Vec<String>)
    -> Result<int, String>
{
    if sup.mode != stop_on_failure {
        fail!("Only stop-on-failure mode implemented");
    }
    {
        args.insert(0, "vagga ".to_string() + cmdname);
        let mut ap = ArgumentParser::new();
        ap.set_description(sup.description.as_ref().map(|x| x.as_slice())
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
    let mut containers = TreeSet::new();
    let mut containers_in_netns = vec!();
    let mut bridges = vec!();
    let mut containers_host_net = vec!();
    let mut forwards = vec!();
    let mut ports = vec!();
    for (name, child) in sup.children.iter() {
        let cont = child.get_container();
        if !containers.contains(cont) {
            containers.insert(cont.to_string());
            match run_wrapper(workdir,
                "_build".to_string(), vec!(cont.to_string()),
                true)
            {
                Ok(0) => {}
                x => return x,
            }
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
    let mut mon = Monitor::new();
    for name in containers_host_net.iter() {
        let mut cmd = Command::new("wrapper".to_string(),
            self_exe_path().unwrap().join("vagga_wrapper"));
        cmd.keep_sigmask();
        cmd.arg(cmdname.as_slice());
        cmd.arg(name.as_slice());
        _common(&mut cmd, workdir);
        cmd.container();
        cmd.set_max_uidmap();
        mon.add(Rc::new(name.clone()), box RunOnce::new(cmd));
    }
    let mut port_forward_guard;
    if containers_in_netns.len() > 0 {
        let gwdir = network::namespace_dir();
        let nsdir = gwdir.join("children");
        if !nsdir.exists() {
            try!(mkdir(&nsdir, ALL_PERMISSIONS)
                .map_err(|e| format!("Failed to create dir: {}", e)));
        }
        try!(network::join_gateway_namespaces());
        try!(unshare_namespace(NewMount)
            .map_err(|e| format!("Failed to create mount namespace: {}", e)));
        try!(mount_tmpfs(&nsdir, "size=10m"));

        let bridge_ns = nsdir.join("bridge");
        let ip = try!(network::setup_bridge(&bridge_ns, &forwards));

        port_forward_guard = network::PortForwardGuard::new(
            gwdir.join("netns"), ip, ports);
        try!(port_forward_guard.start_forwarding());

        for name in containers_in_netns.iter() {
            let child = sup.children.find(name).unwrap();
            let mut cmd = Command::new("wrapper".to_string(),
                self_exe_path().unwrap().join("vagga_wrapper"));
            cmd.keep_sigmask();
            cmd.arg(cmdname.as_slice());
            cmd.arg(name.as_slice());
            _common(&mut cmd, workdir);
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
                net_ns = nsdir.join("net.".to_string() + netw.ip.as_slice());
                uts_ns = nsdir.join("uts.".to_string() + netw.ip.as_slice());
                // TODO(tailhook) support multiple commands with same IP
                try!(network::setup_container(&net_ns, &uts_ns,
                    name.as_slice(), netw.ip.as_slice(),
                    netw.hostname.as_ref().unwrap_or(name).as_slice()));
                try!(set_namespace(&net_ns, NewNet)
                    .map_err(|e| format!("Error setting netns: {}", e)));
                try!(set_namespace(&uts_ns, NewUts)
                    .map_err(|e| format!("Error setting netns: {}", e)));
            }

            let name = Rc::new(name.clone());
            mon.add(name.clone(), box RunOnce::new(cmd));
            try!(mon.force_start(name));
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
