use crate::config::command::{MainCommand, CommandInfo, SuperviseInfo, CapsuleInfo};
use crate::launcher::{Context, prerequisites};
use crate::launcher::{capsule, simple, supervisor, system};


enum Args<'a> {
    Simple(&'a CommandInfo, simple::Args),
    Capsule(&'a CapsuleInfo, capsule::Args),
    Supervise(&'a SuperviseInfo, supervisor::Args),
}

enum Data<'a> {
    Simple(&'a CommandInfo, simple::Args, simple::Version),
    Capsule(&'a CapsuleInfo, capsule::Args, capsule::Data),
    Supervise(&'a SuperviseInfo, supervisor::Args, supervisor::Data),
}

#[cfg(feature="containers")]
pub fn run_user_command(context: &Context, cmd: String, args: Vec<String>)
    -> Result<i32, String>
{
    run_commands(context, vec![cmd], args)
}

#[cfg(feature="containers")]
pub fn run_multiple_commands(context: &Context, commands: Vec<String>)
    -> Result<i32, String>
{
    run_commands(context, commands, Vec::new())
}

#[cfg(feature="containers")]
fn run_commands(context: &Context, commands: Vec<String>,
    last_command_args: Vec<String>)
    -> Result<i32, String>
{
    let mut commands = commands.into_iter().map(|name| {
        if !context.config.commands.contains_key(&name) {
            for (cname, cmd) in &context.config.commands {
                if cmd.aliases().contains(&name) {
                    return Ok(cname.to_string());
                }
            }
            return Err(format!("Command {:?} not found and is not an alias",
                               name));
        } else {
            Ok(name)
        }
    }).collect::<Result<_, _>>()?;
    if context.prerequisites {
        commands = prerequisites::scan(context, commands);
    }

    let mut all_args = Vec::new();
    let last_cmd = commands.len() -1;
    let mut last_cmd_args = Some(last_command_args);
    let iter = commands.into_iter().enumerate().map(|(i, x)| {
            (x,
             if i == last_cmd {
                 last_cmd_args.take().unwrap()
             } else {
                    Vec::new()
             })
        });
    for (cmd, args) in iter {
        let cinfo = match context.config.commands.get(&cmd) {
            Some(x) => x,
            None => return Err(format!("Command {} not found. \
                        Run vagga without arguments to see the list.", cmd)),
        };
        system::check(&cinfo.system(), context)?;

        use crate::launcher::options::ArgError::*;

        let arg = match *cinfo {
            MainCommand::Command(ref info) => {
                let a = match simple::parse_args(info, context, cmd, args) {
                    Ok(a) => a,
                    Err(Exit(x)) => return Ok(x),
                    Err(Error(e)) => return Err(e),
                };
                Args::Simple(info, a)
            }
            MainCommand::CapsuleCommand(ref info) => {
                let a = match capsule::parse_args(info, context, cmd, args) {
                    Ok(a) => a,
                    Err(Exit(x)) => return Ok(x),
                    Err(Error(e)) => return Err(e),
                };
                Args::Capsule(info, a)
            }
            MainCommand::Supervise(ref info) => {
                let a = match supervisor::parse_args(info, context, cmd, args)
                {
                    Ok(a) => a,
                    Err(Exit(x)) => return Ok(x),
                    Err(Error(e)) => return Err(e),
                };
                Args::Supervise(info, a)
            }
        };
        all_args.push(arg);
    }
    let mut datas = Vec::new();
    for arg in all_args.into_iter() {
        let data = match arg {
            Args::Simple(info, arg) => {
                let v = simple::prepare_containers(info, &arg, context)?;
                Data::Simple(info, arg, v)
            }
            Args::Capsule(info, arg) => {
                let v = capsule::prepare_containers(info, &arg, context)?;
                Data::Capsule(info, arg, v)
            }
            Args::Supervise(info, arg) => {
                let v = supervisor::prepare_containers(info, &arg, context)?;
                Data::Supervise(info, arg, v)
            }
        };
        datas.push(data);
    }
    if context.containers_only {
        debug!("Containers are prepared. Ready to exit.");
        return Ok(0);
    }
    for data in datas.into_iter() {
        let result = match data {
            Data::Simple(info, arg, data) => {
                simple::run(info, arg, data, context)
            }
            Data::Capsule(info, arg, data) => {
                capsule::run(info, arg, data, context)
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
