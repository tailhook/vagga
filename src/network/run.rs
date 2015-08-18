use std::env;
use std::io::{stdout, stderr};
use std::path::Path;
use std::process::{Command, Stdio};

use argparse::{ArgumentParser, Store, List};

use config::Config;
use config::command::{Networking};
use config::command::{MainCommand};
use container::nsutil::set_namespace;
use container::container::Namespace::NewNet;


pub fn run_command_cmd(config: &Config, args: Vec<String>)
    -> Result<(), Result<i32, String>>
{
    let mut subcommand = "".to_string();
    let mut command = "".to_string();
    let mut cmdargs = Vec::<String>::new();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Runs command in specified container's network namespace.
            The command runs in current mount namespace (i.e. same file system)
            ");
        ap.refer(&mut subcommand)
            .add_argument("node", Store,
                "A node (subcommand) which namespace to run in");
        ap.refer(&mut command)
            .add_argument("command", Store,
                "A command to run in namespace");
        ap.refer(&mut cmdargs)
            .add_argument("args", List,
                "Additional arguments to command");
        ap.stop_on_first_argument(true);
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Err(Ok(0)),
            Err(x) => {
                return Err(Ok(x));
            }
        }
    }
    let cmd = try!(env::var("VAGGA_COMMAND").ok()
        .and_then(|cmd| config.commands.get(&cmd))
        .ok_or(Err(format!("This command is supposed to be run inside \
                        container started by vagga !Supervise command"))));
    let sup = match cmd {
        &MainCommand::Supervise(ref sup) => sup,
        _ => return Err(Err(format!("This command is supposed to be run \
                inside container started by vagga !Supervise command"))),
    };
    let ip = if let Some(child) = sup.children.get(&subcommand) {
        if let Some(ref netw) = child.network() {
            netw.ip.clone()
        } else {
            return Err(Err(format!("Node {} does not have IP", subcommand)));
        }
    } else {
        return Err(Err(format!("Node {} is missing", subcommand)));
    };
    try!(set_namespace(
        &Path::new(&format!("/tmp/vagga/namespaces/net.{}", ip)), NewNet)
        .map_err(|e| Err(format!("Can't set namespace: {}", e))));

    let mut cmd = Command::new(&command);
    cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    cmd.args(&cmdargs);
    match cmd.status() {
        Ok(status) if status.success() => Ok(()),
        e => Err(Err(format!("Error running {}: {:?}", command, e))),
    }
}

