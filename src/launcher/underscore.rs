use std::io::{stdout, stderr};

use argparse::{ArgumentParser};
use argparse::{StoreTrue, List, StoreOption, Store};
use unshare::{Command, Namespace};

use options::build_mode::build_mode;
use options::version_hash;
use container::nsutil::{set_namespace};
use process_util::{run_and_wait, convert_status};

use super::network;
use super::build::{build_container};
use super::wrap::Wrapper;
use launcher::Context;
use launcher::volumes::prepare_volumes;


pub fn run_command(context: &Context, mut args: Vec<String>)
    -> Result<i32, String>
{
    let mut cmdargs = Vec::<String>::new();
    let mut container = "".to_string();
    let mut command = "".to_string();
    let mut copy = false;
    let mut bmode = context.build_mode;
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
    let cinfo = try!(context.config.get_container(&container));
    let ver = try!(build_container(context, &container, bmode));
    try!(prepare_volumes(cinfo.volumes.values(), context));
    let mut cmd: Command = Wrapper::new(Some(&ver), &context.settings);
    cmd.workdir(&context.workdir);
    cmd.arg("_run");
    cmd.args(&args[1..]);
    try!(cmd.map_users_for(cinfo, &context.settings));
    cmd.gid(0);
    cmd.groups(Vec::new());
    let res = run_and_wait(&mut cmd).map(convert_status);

    if copy {
        let mut cmd: Command = Wrapper::new(None, &context.settings);
        cmd.workdir(&context.workdir);  // TODO(tailhook) why is it needed?
        cmd.max_uidmap();
        cmd.gid(0);
        cmd.groups(Vec::new());
        cmd.arg("_clean").arg("--transient");
        match cmd.status() {
            Ok(s) if s.success() => {}
            Ok(s) => warn!("The `vagga _clean --transient` {}", s),
            Err(e) => warn!("Failed to run `vagga _clean --transient`: {}", e),
        }

    }
    return res;
}

pub fn run_in_netns(context: &Context, cname: String, mut args: Vec<String>)
    -> Result<i32, String>
{
    let mut cmdargs: Vec<String> = vec!();
    let mut container = "".to_string();
    let mut pid = None;
    let mut bmode = context.build_mode;
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
    let ver = try!(build_container(context, &container, bmode));
    try!(network::join_gateway_namespaces());
    if let Some::<i32>(pid) = pid {
        try!(set_namespace(format!("/proc/{}/ns/net", pid), Namespace::Net)
            .map_err(|e| format!("Error setting networkns: {}", e)));
    }
    let mut cmd: Command = Wrapper::new(Some(&ver), &context.settings);
    cmd.workdir(&context.workdir);
    cmd.arg(cname);
    cmd.arg(container.clone());
    cmd.args(&cmdargs);
    run_and_wait(&mut cmd).map(convert_status)
}

pub fn version_hash(ctx: &Context, cname: &str, mut args: Vec<String>)
    -> Result<i32, String>
{
    args.insert(0, "vagga _version_hash".to_string());
    let opt = match version_hash::Options::parse(&args, false) {
        Ok(x) => x,
        Err(e) => return Ok(e),
    };
    let mut cmd: Command = Wrapper::new(None, &ctx.settings);
    cmd.workdir(&ctx.workdir);
    try!(cmd.map_users_for(
        &ctx.config.get_container(&opt.container).unwrap(),
        &ctx.settings));
    cmd.gid(0);
    cmd.groups(Vec::new());
    cmd.arg(&cname).args(&args[1..]);
    cmd.status()
    .map(convert_status)
    .map_err(|e| format!("Error running `vagga_wrapper {}`: {}",
                         cname, e))
}
pub fn passthrough(ctx: &Context, cname: &str, args: Vec<String>)
    -> Result<i32, String>
{
    let mut cmd: Command = Wrapper::new(None, &ctx.settings);
    cmd.workdir(&ctx.workdir);
    cmd.max_uidmap();
    cmd.gid(0);
    cmd.groups(Vec::new());
    cmd.arg(&cname).args(&args);
    cmd.status()
    .map(convert_status)
    .map_err(|e| format!("Error running `vagga_wrapper {}`: {}",
                         cname, e))
}
