use std::io::{stdout, stderr};
use libc::pid_t;

use argparse::{ArgumentParser};
use argparse::{StoreTrue, StoreFalse, List, StoreOption, Store};

use config::command::{CommandInfo, Networking};
use container::container::Namespace::{NewUser, NewNet};
use container::nsutil::{set_namespace};

use super::user;
use super::network;


pub fn run_command(workdir: &Path, cmdname: String, mut args: Vec<String>)
    -> Result<isize, String>
{
    let mut cmdargs = vec!();
    let mut container = "".to_string();
    let mut pid = None;
    {
        args.insert(0, "vagga ".to_string() + cmdname.as_slice());
        let mut ap = ArgumentParser::new();
        ap.set_description(
            "Run command (or shell) in one of the vagga's network namespaces");
        ap.refer(&mut pid)
            .add_option(&["--pid"], Box::new(StoreOption::<pid_t>), "
                Run in the namespace of the process with PID.
                By default you get shell in the \"gateway\" namespace.
                ");
        ap.refer(&mut container)
            .add_argument("container", Box::new(Store::<String>),
                "Container to run command in")
            .required();
        ap.refer(&mut cmdargs)
            .add_argument("command", Box::new(List::<String>),
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
    match user::run_wrapper(workdir, "_build".to_string(),
        vec!(container), true)
    {
        Ok(0) => {}
        x => return x,
    }
    user::run_wrapper(workdir, cmdname, cmdargs, true)
}

pub fn run_in_netns(workdir: &Path, cname: String, mut args: Vec<String>)
    -> Result<isize, String>
{
    let mut cmdargs = vec!();
    let mut container = "".to_string();
    let mut pid = None;
    {
        args.insert(0, "vagga ".to_string() + cname.as_slice());
        let mut ap = ArgumentParser::new();
        ap.set_description(
            "Run command (or shell) in one of the vagga's network namespaces");
        ap.refer(&mut pid)
            .add_option(&["--pid"], Box::new(StoreOption::<pid_t>), "
                Run in the namespace of the process with PID.
                By default you get shell in the \"gateway\" namespace.
                ");
        ap.refer(&mut container)
            .add_argument("container", Box::new(Store::<String>),
                "Container to run command in")
            .required();
        ap.refer(&mut cmdargs)
            .add_argument("command", Box::new(List::<String>),
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

    match user::run_wrapper(workdir, "_build".to_string(),
        vec!(container), true)
    {
        Ok(0) => {}
        x => return x,
    }

    try!(network::join_gateway_namespaces());
    if let Some(pid) = pid {
        try!(set_namespace(&Path::new(format!("/proc/{}/ns/net", pid)), NewNet)
            .map_err(|e| format!("Error setting networkns: {}", e)));
    }
    user::run_wrapper(workdir, cname, cmdargs, false)
}
