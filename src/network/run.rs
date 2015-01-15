use std::os::getenv;
use std::io::{stdout, stderr};
use std::io::process::{Command, InheritFd, ExitStatus};

use argparse::{ArgumentParser, Store, List};

use config::Config;
use config::command::{Networking};
use config::command::main;
use container::nsutil::set_namespace;
use container::container::NewNet;


pub fn run_command_cmd(config: &Config, args: Vec<String>)
    -> Result<(), Result<int, String>>
{
    let mut subcommand = "".to_string();
    let mut command = "".to_string();
    let mut cmdargs = vec!();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Runs command in specified container's network namespace.
            The command runs in current mount namespace (i.e. same file system)
            ");
        ap.refer(&mut subcommand)
            .add_argument("node", box Store::<String>,
                "A node (subcommand) which namespace to run in");
        ap.refer(&mut command)
            .add_argument("command", box Store::<String>,
                "A command to run in namespace");
        ap.refer(&mut cmdargs)
            .add_argument("args", box List::<String>,
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
    let cmd = try!(getenv("VAGGA_COMMAND")
        .and_then(|cmd| config.commands.find(&cmd))
        .ok_or(Err(format!("This command is supposed to be run inside \
                        container started by vagga !Supervise command"))));
    let sup = match cmd {
        &main::Supervise(ref sup) => sup,
        _ => return Err(Err(format!("This command is supposed to be run \
                inside container started by vagga !Supervise command"))),
    };
    let ip = if let Some(child) = sup.children.find(&subcommand) {
        if let Some(ref netw) = child.network() {
            netw.ip.clone()
        } else {
            return Err(Err(format!("Node {} does not have IP", subcommand)));
        }
    } else {
        return Err(Err(format!("Node {} is missing", subcommand)));
    };
    try!(set_namespace(
        &Path::new(format!("/tmp/vagga/namespaces/net.{}", ip)), NewNet)
        .map_err(|e| Err(format!("Can't set namespace: {}", e))));

    let mut cmd = Command::new(command.as_slice());
    cmd.stdout(InheritFd(1)).stderr(InheritFd(2));
    cmd.args(cmdargs.as_slice());
    match cmd.status() {
        Ok(ExitStatus(0)) => Ok(()),
        e => Err(Err(format!("Error running {}: {}", command, e))),
    }
}

