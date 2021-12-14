use std::collections::HashMap;

use unshare::{Command, Fd};

use crate::config::command::{CommandInfo, WriteMode};
use crate::launcher::{Context, socket};
use crate::launcher::options::{ArgError, parse_docopts};
use crate::launcher::volumes::prepare_volumes;
use crate::process_util::{run_and_wait, convert_status};

use super::build::{build_container};
use super::network;
use super::wrap::Wrapper;


const DEFAULT_DOCOPT: &'static str = "\
Common options:
  -h, --help           This help
";


pub type Version = String;

pub struct Args {
    cmdname: String,
    args: Vec<String>,
    environ: HashMap<String, String>,
}


pub fn parse_args(cinfo: &CommandInfo, _context: &Context,
    cmd: String, args: Vec<String>)
    -> Result<Args, ArgError>
{
    if let Some(_) = cinfo.network {
        return Err(ArgError::Error(format!(
            "Network is not supported for !Command use !Supervise")));
    }
    if let Some(ref opttext) = cinfo.options {
        let (env, _) = parse_docopts(&cinfo.description, opttext,
                                          DEFAULT_DOCOPT,
                                          &cmd, args)?;
        Ok(Args {
            cmdname: cmd,
            environ: env,
            args: Vec::new(),
        })
    } else {
        Ok(Args {
            cmdname: cmd,
            environ: HashMap::new(),
            args: args,
        })
    }
}

pub fn prepare_containers(cinfo: &CommandInfo, _: &Args, context: &Context)
    -> Result<Version, String>
{
    let ver = build_container(context, &cinfo.container,
        context.build_mode, false)?;
    let cont = context.config.containers.get(&cinfo.container)
        .ok_or_else(|| format!("Container {:?} not found", cinfo.container))?;
    prepare_volumes(cont.volumes.values(), context)?;
    prepare_volumes(cinfo.volumes.values(), context)?;
    return Ok(ver);
}

pub fn run(cinfo: &CommandInfo, args: Args, version: Version,
    context: &Context)
    -> Result<i32, String>
{
    if cinfo.isolate_network || context.isolate_network {
        try_msg!(network::isolate_network(),
            "Cannot setup isolated network: {err}");
    }

    let mut cmd: Command = Wrapper::new(Some(&version), &context.settings);
    cmd.workdir(&context.workdir);
    for (k, v) in &args.environ {
        cmd.env(k, v);
    }
    cmd.arg(&args.cmdname);
    cmd.args(&args.args);
    if let Some(ref sock_str) = cinfo.pass_tcp_socket {
        let sock = socket::parse_and_bind(sock_str)
            .map_err(|e| format!("Error listening {:?}: {}", sock_str, e))?;
        cmd.file_descriptor(3, Fd::from_file(sock));
    }
    if cinfo.network.is_none() { // TODO(tailhook) is it still a thing?
        cmd.map_users_for(
            &context.config.get_container(&cinfo.container).unwrap(),
            &context.settings)?;
        cmd.gid(0);
        cmd.groups(Vec::new());
    }
    let res = run_and_wait(&mut cmd).map(convert_status);

    if cinfo.write_mode != WriteMode::read_only {
        let mut cmd: Command = Wrapper::new(None, &context.settings);
        cmd.workdir(&context.workdir);
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
    if res == Ok(0) {
        if let Some(ref epilog) = cinfo.epilog {
            print!("{}", epilog);
        }
    }
    return res;
}
