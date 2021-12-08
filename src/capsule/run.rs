use std::io::{stdout, stderr};

use argparse::{ArgumentParser};
use argparse::{List, Store};
use unshare::{Command};

use crate::capsule::Context;
use crate::launcher::wrap::Wrapper;
use crate::launcher::build::build_container;
use crate::process_util::{run_and_wait, convert_status};


pub fn run_command(context: &Context, mut args: Vec<String>)
    -> Result<i32, String>
{
    let mut cmdargs = Vec::<String>::new();
    let mut container = "".to_string();
    let mut command = "".to_string();
    {
        args.insert(0, "vagga _capsule run".to_string());
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Runs arbitrary command inside the container
            ");
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
    // TODO(tailhook) build container
    // TODO(tailhook) prepare volumes
    // TODO(tailhook) network isolation?

    let ver = build_container(context, &container, context.build_mode, true)?;
    debug!("Container {:?} is built with version {:?}", container, ver);

    let mut cmd: Command = Wrapper::new(Some(&ver), &context.settings);
    cmd.workdir(&context.workdir);
    cmd.arg("_run");
    cmd.args(&args[1..]);
    /* TODO(tailhook) check if uid map works
    cmd.map_users_for(cinfo, &context.settings)?;
    */
    cmd.gid(0);
    cmd.groups(Vec::new());
    let res = run_and_wait(&mut cmd).map(convert_status);

    // TODO(tailhook) clean transient container ?

    return res;
}
