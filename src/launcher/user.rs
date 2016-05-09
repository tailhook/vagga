use config::command::{MainCommand, CommandInfo, SuperviseInfo};
use launcher::{supervisor, simple};
use launcher::Context;


pub enum ArgError {
    Exit(i32),
    Error(String),
}

enum Args<'a> {
    Simple(&'a CommandInfo, simple::Args),
    Supervise(&'a SuperviseInfo, supervisor::Args),
}

enum Data<'a> {
    Simple(&'a CommandInfo, simple::Args, simple::Version),
    Supervise(&'a SuperviseInfo, supervisor::Args, supervisor::Data),
}

pub fn run_user_command(context: &Context, cmd: String, args: Vec<String>)
    -> Result<i32, String>
{
    use self::ArgError::*;
    match context.config.commands.get(&cmd) {
        None => Err(format!("Command {} not found. \
                    Run vagga without arguments to see the list.", cmd)),
        Some(&MainCommand::Command(ref info)) => {
            let a = match simple::parse_args(info, context, cmd, args) {
                Ok(a) => a,
                Err(Exit(x)) => return Ok(x),
                Err(Error(e)) => return Err(e),
            };
            // TODO(tailhook) prereq containers
            let v = try!(simple::prepare_containers(info, &a, context));
            // TODO(tailhook) prereq commands
            simple::run(info, a, v, context)
        }
        Some(&MainCommand::Supervise(ref info)) => {
            let a = match supervisor::parse_args(info, context, cmd, args) {
                Ok(a) => a,
                Err(Exit(x)) => return Ok(x),
                Err(Error(e)) => return Err(e),
            };
            // TODO(tailhook) prereq containers
            let v = try!(supervisor::prepare_containers(info, &a, context));
            // TODO(tailhook) prereq commands
            supervisor::run(info, a, v, context)
        }
    }
}

pub fn run_multiple_commands(context: &Context, commands: Vec<String>)
    -> Result<i32, String>
{
    use self::ArgError::*;
    let mut args = Vec::new();
    for cmd in commands.into_iter() {
        let arg = match context.config.commands.get(&cmd) {
            None => return Err(format!("Command {} not found. \
                        Run vagga without arguments to see the list.", cmd)),
            Some(&MainCommand::Command(ref info)) => {
                let a = match simple::parse_args(info, context,
                                                 cmd, Vec::new())
                {
                    Ok(a) => a,
                    Err(Exit(x)) => return Ok(x),
                    Err(Error(e)) => return Err(e),
                };
                Args::Simple(info, a)
            }
            Some(&MainCommand::Supervise(ref info)) => {
                let a = match supervisor::parse_args(info, context,
                                                     cmd, Vec::new())
                {
                    Ok(a) => a,
                    Err(Exit(x)) => return Ok(x),
                    Err(Error(e)) => return Err(e),
                };
                Args::Supervise(info, a)
            }
        };
        args.push(arg);
    }
    let mut datas = Vec::new();
    for arg in args.into_iter() {
        let data = match arg {
            Args::Simple(info, arg) => {
                let v = try!(simple::prepare_containers(
                    info, &arg, context));
                Data::Simple(info, arg, v)
            }
            Args::Supervise(info, arg) => {
                let v = try!(supervisor::prepare_containers(
                    info, &arg, context));
                Data::Supervise(info, arg, v)
            }
        };
        datas.push(data);
    }
    for data in datas.into_iter() {
        let result = match data {
            Data::Simple(info, arg, data) => {
                simple::run(info, arg, data, context)
            }
            Data::Supervise(info, arg, data) => {
                supervisor::run(info, arg, data, context)
            }
        };
        match result {
            Ok(0) => continue,
            other => return other,
        }
    }
    Ok(0)
}
