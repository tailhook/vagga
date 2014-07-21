use std::io::stdio::{stdout, stderr};

use collections::treemap::TreeMap;
use argparse::ArgumentParser;

use super::super::env::Environ;
use super::super::options::env_options;
use super::super::run::internal_run;
use super::super::config::Shell;


pub fn run_shell_command(env: &mut Environ, cmdname: &String,
    args: Vec<String>)
    -> Result<int, String>
{
    let has_arguments;
    let description;
    {
        let command = match env.config.commands.find(cmdname) {
            Some(c) => c,
            None => {
                return Err(format!("Can't find command {} in config",
                                   cmdname));
            }
        };
        has_arguments = command.accepts_arguments;
        description = command.description.clone().unwrap_or("".to_string());
    }
    let mut cmdargs;
    if has_arguments {
        //  All options forwarded to command (including --help and others)
        cmdargs = args;
        cmdargs.shift();  // Zeroth arg is a command
    } else {
        //  We can provide useful help in this case
        cmdargs = Vec::new();
        let mut ap = ArgumentParser::new();
        if description.len() > 0 {
            ap.set_description(description.as_slice());
        }
        env_options(env, &mut ap);
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => return Ok(122),
        }
    }
    let command = env.config.commands.find(cmdname).unwrap();
    let cname = match command.container {
        Some(ref name) => name.clone(),
        None => unimplemented!(),
    };
    let container = try!(env.get_container(&cname));
    let mut runenv = TreeMap::new();
    for (k, v) in command.environ.iter() {
        runenv.insert(k.clone(), v.clone());
    }
    let mut argprefix: Vec<String> = Vec::new();
    argprefix.extend(container.shell.clone().move_iter());
    match command.execute {
        Shell(ref x) => argprefix.push(x.clone()),
        _ => unreachable!(),
    }
    let cmd = argprefix.shift().unwrap();
    return internal_run(env, container,
        cmd, (argprefix + cmdargs), runenv);
}
