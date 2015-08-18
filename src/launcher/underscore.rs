use std::io::{stdout, stderr};
use std::path::Path;

use argparse::{ArgumentParser};
use argparse::{StoreTrue, List, StoreOption, Store};

use config::Config;
use container::container::Namespace::{NewNet};
use container::nsutil::{set_namespace};

use super::user;
use super::network;
use super::build::build_container;


pub fn run_command(config: &Config, workdir: &Path, cmdname: String,
    mut args: Vec<String>)
    -> Result<i32, String>
{
    let mut cmdargs = Vec::<String>::new();
    let mut container = "".to_string();
    let mut copy = false;
    {
        args.insert(0, "vagga ".to_string() + &cmdname);
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
        ap.refer(&mut container)
            .add_argument("container", Store,
                "Container to run command in")
            .required();
        ap.refer(&mut cmdargs)
            .add_argument("command", List,
                "Command (with arguments) to run inside container")
            .required();

        ap.stop_on_first_argument(true);
        match ap.parse(args.clone(), &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => {
                return Ok(122);
            }
        }
    }
    args.remove(0);
    try!(build_container(config, &container));
    let res = user::run_wrapper(Some(workdir), cmdname, args, true);

    if copy {
        match user::run_wrapper(Some(workdir), "_clean".to_string(),
            vec!("--transient".to_string()), true)
        {
            Ok(0) => {}
            x => warn!(
                "The `vagga _clean --transient` exited with status: {:?}", x),
        }

    }
    return res;
}

pub fn run_in_netns(config: &Config, workdir: &Path, cname: String,
    mut args: Vec<String>)
    -> Result<i32, String>
{
    let mut cmdargs = vec!();
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
    cmdargs.insert(0, container.clone());
    try!(build_container(config, &container));
    try!(network::join_gateway_namespaces());
    if let Some::<i32>(pid) = pid {
        try!(set_namespace(
                &Path::new(&format!("/proc/{}/ns/net", pid)), NewNet)
            .map_err(|e| format!("Error setting networkns: {}", e)));
    }
    user::run_wrapper(Some(workdir), cname, cmdargs, false)
}
