use unshare::{Command, Fd};

use config::command::{CommandInfo, Networking, WriteMode};
use process_util::{run_and_wait, convert_status};
use super::build::{build_container};
use super::wrap::Wrapper;
use launcher::volumes::prepare_volumes;
use launcher::user::ArgError;
use launcher::Context;
use launcher::socket;

pub type Args = (String, Vec<String>);
pub type Version = String;


pub fn parse_args(cinfo: &CommandInfo, _context: &Context,
    cmd: String, args: Vec<String>)
    -> Result<Args, ArgError>
{
    if let Some(_) = cinfo.network {
        return Err(ArgError::Error(format!(
            "Network is not supported for !Command use !Supervise")));
    }
    Ok((cmd, args))
}

pub fn prepare_containers(cinfo: &CommandInfo, _: &Args, context: &Context)
    -> Result<Version, String>
{
    let ver = try!(build_container(
        &context.settings, &cinfo.container, context.build_mode));
    let cont = try!(context.config.containers.get(&cinfo.container)
        .ok_or_else(|| format!("Container {:?} not found", cinfo.container)));
    try!(prepare_volumes(cont.volumes.values(), context));
    try!(prepare_volumes(cinfo.volumes.values(), context));
    return Ok(ver);
}

pub fn run(cinfo: &CommandInfo, (cmdname, args): Args, version: Version,
    context: &Context)
    -> Result<i32, String>
{
    let mut cmd: Command = Wrapper::new(Some(&version), &context.settings);
    cmd.workdir(&context.workdir);
    cmd.arg(cmdname);
    cmd.args(&args);
    if let Some(ref sock_str) = cinfo.pass_tcp_socket {
        let sock = try!(socket::parse_and_bind(sock_str)
            .map_err(|e| format!("Error listening {:?}: {}", sock_str, e)));
        cmd.file_descriptor(3, Fd::from_file(sock));
    }
    if cinfo.network.is_none() { // TODO(tailhook) is it still a thing?
        cmd.userns();
    }
    let res = run_and_wait(&mut cmd).map(convert_status);

    if cinfo.write_mode != WriteMode::read_only {
        let mut cmd: Command = Wrapper::new(None, &context.settings);
        cmd.workdir(&context.workdir);
        cmd.userns();
        cmd.arg("_clean").arg("--transient");
        match cmd.status() {
            Ok(s) if s.success() => {}
            Ok(s) => warn!("The `vagga _clean --transient` {}", s),
            Err(e) => warn!("Failed to run `vagga _clean --transient`: {}", e),
        }

    }
    if res == Ok(0) {
        if let Some(ref epilog) = cinfo.epilog {
            print!("{}", epilog);
        }
    }
    return res;
}
