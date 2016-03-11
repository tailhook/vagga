use std::io::{stdout, stderr};
use std::path::Path;

use argparse::{ArgumentParser};
use argparse::{StoreTrue, List, StoreOption, Store};
use unshare::{Command, Namespace};

use options::build_mode::{build_mode, BuildMode};
use config::{Settings};
use container::nsutil::{set_namespace};

use super::network;
use super::build::{build_container};
use super::wrap::Wrapper;


pub fn run_command(settings: &Settings, workdir: &Path,
    mut args: Vec<String>, mut bmode: BuildMode)
    -> Result<i32, String>
{
    let mut cmdargs = Vec::<String>::new();
    let mut container = "".to_string();
    let mut command = "".to_string();
    let mut copy = false;
    {
        args.insert(0, "vagga _run".to_string());
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Runs arbitrary command inside the container
            ");
        ap.refer(&mut copy)
            .add_option(&["-W", "--writeable"], StoreTrue,
                "Create translient writeable container for running the command.
                 Currently we use hard-linked copy of the container, so it's
                 dangerous for some operations. Still it's ok for installing
                 packages or similar tasks");
        build_mode(&mut ap, &mut bmode);
        ap.refer(&mut container)
            .add_argument("container", Store,
                "Container to run command in")
            .required();
        ap.refer(&mut command)
            .add_argument("command", Store,
                "Command to run inside the container")
            .required();
        ap.refer(&mut cmdargs)
            .add_argument("arg", List, "Arguments to the command");

        ap.stop_on_first_argument(true);
        match ap.parse(args.clone(), &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => {
                return Ok(122);
            }
        }
    }
    let ver = try!(build_container(settings, &container, bmode, None));
    let mut cmd: Command = Wrapper::new(Some(&ver), settings);
    cmd.workdir(workdir);
    cmd.arg("_run");
    cmd.args(&args[1..]);
    cmd.userns();
    let res = cmd.run();

    if copy {
        let mut cmd: Command = Wrapper::new(None, settings);
        cmd.workdir(workdir);
        cmd.userns();
        cmd.arg("_clean").arg("--transient");
        match cmd.run() {
            Ok(0) => {}
            x => warn!(
                "The `vagga _clean --transient` exited with status: {:?}", x),
        }

    }
    return res;
}

pub fn run_in_netns(settings: &Settings, workdir: &Path, cname: String,
    mut args: Vec<String>, mut bmode: BuildMode)
    -> Result<i32, String>
{
    let mut cmdargs: Vec<String> = vec!();
    let mut container = "".to_string();
    let mut pid = None;
    {
        args.insert(0, "vagga ".to_string() + &cname);
        let mut ap = ArgumentParser::new();
        ap.set_description(
            "Run command (or shell) in one of the vagga's network namespaces");
        ap.refer(&mut pid)
            .add_option(&["--pid"], StoreOption, "
                Run in the namespace of the process with PID.
                By default you get shell in the \"gateway\" namespace.
                ");
        build_mode(&mut ap, &mut bmode);
        ap.refer(&mut container)
            .add_argument("container", Store,
                "Container to run command in")
            .required();
        ap.refer(&mut cmdargs)
            .add_argument("command", List,
                "Command (with arguments) to run inside container")
            .required();

        ap.stop_on_first_argument(true);
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => {
                return Ok(122);
            }
        }
    }
    let ver = try!(build_container(settings, &container, bmode, None));
    try!(network::join_gateway_namespaces());
    if let Some::<i32>(pid) = pid {
        try!(set_namespace(format!("/proc/{}/ns/net", pid), Namespace::Net)
            .map_err(|e| format!("Error setting networkns: {}", e)));
    }
    let mut cmd: Command = Wrapper::new(Some(&ver), settings);
    cmd.workdir(workdir);
    cmd.arg(cname);
    cmd.arg(container.clone());
    cmd.args(&cmdargs);
    cmd.run()
}
