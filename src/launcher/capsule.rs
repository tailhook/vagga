use std::collections::HashMap;

use unshare::{Command, Fd};

use config::command::{CapsuleInfo};
use process_util::{run_and_wait, convert_status};
use super::wrap::Wrapper;
use super::network;
use launcher::user::ArgError;
use launcher::Context;
use launcher::socket;
use launcher::options::parse_docopts;


const DEFAULT_DOCOPT: &'static str = "\
Common options:
  -h, --help           This help
";


pub type Data = ();

pub struct Args {
    cmdname: String,
    args: Vec<String>,
    environ: HashMap<String, String>,
}


pub fn parse_args(cinfo: &CapsuleInfo, _context: &Context,
    cmd: String, args: Vec<String>)
    -> Result<Args, ArgError>
{
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

pub fn prepare_containers(_: &CapsuleInfo, _: &Args, _: &Context)
    -> Result<(), String>
{
    Ok(())
}

pub fn run(cinfo: &CapsuleInfo, args: Args, _data: Data, context: &Context)
    -> Result<i32, String>
{
    if cinfo.isolate_network || context.isolate_network {
        try_msg!(network::isolate_network(),
            "Cannot setup isolated network: {err}");
    }

    let mut cmd: Command = Wrapper::new(None, &context.settings);
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
    cmd.map_users(&cinfo.uids, &cinfo.gids, &context.settings)?;
    cmd.gid(0);
    cmd.groups(Vec::new());
    let res = run_and_wait(&mut cmd).map(convert_status);

    if res == Ok(0) {
        if let Some(ref epilog) = cinfo.epilog {
            print!("{}", epilog);
        }
    }
    return res;
}
