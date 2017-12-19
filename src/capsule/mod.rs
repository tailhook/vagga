mod build;
mod run;
mod script;
pub mod packages;
pub mod download;

use std::io::{Write, stdout, stderr};

use argparse::{ArgumentParser, Store, List};

use launcher;

pub use self::packages::State;
pub type Context = launcher::Context;


pub fn run_command(context: &Context, mut input_args: Vec<String>)
    -> Result<i32, String>
{
    let mut args = Vec::<String>::new();
    let mut cname = "".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Run a special capsule command. Only can be run from inside the
            `!CapsuleCommand` handler.
            ");
        ap.refer(&mut cname)
          .add_argument("command", Store,
                "A command to run: only `build` supported so far");
        ap.refer(&mut args)
          .add_argument("args", List,
                "Arguments for the command");
        ap.stop_on_first_argument(true);
        input_args.insert(0, String::from("vagga _capsule"));
        match ap.parse(input_args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => return Ok(122),
        }
    }
    match &cname[..] {
        "build" => build::build_command(context, args),
        "run" => run::run_command(context, args),
        "script" => script::run_script(context, args),
        "download" => download::run_download(context, args),
        _ => {
            writeln!(&mut stderr(), "Unknown command {:?}", cname).ok();
            Ok(127)
        }
    }
}
