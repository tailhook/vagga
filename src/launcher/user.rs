use config::command::MainCommand;
use launcher::{supervisor, simple};
use launcher::Context;


pub enum ArgError {
    Exit(i32),
    Error(String),
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

