use argparse::{ArgumentParser, StoreOption, List};
use std::os::{getcwd, args};
use std::io::stdio::stderr;

use super::config::find_config;
use super::build::build_command;
use super::run::{run_command, run_user_command};
use super::env::Environ;
use super::options::env_options;

pub fn run() -> int {
    let mut err = stderr();
    let workdir = getcwd();

    let vcmd = args().move_iter().next().unwrap();
    let mypath = Path::new(vcmd.as_slice());

    let (config, project_root) = match find_config(&workdir) {
        Ok(tup) => tup,
        Err(e) => {
            err.write_line(e.as_slice()).ok();
            return 126;
        }
    };
    let mut env = Environ {
        vagga_dir: mypath.dir_path(),
        vagga_path: mypath,
        vagga_command: vcmd.clone(),
        work_dir: workdir,
        project_root: project_root,
        variables: Vec::new(),
        config: config,
    };

    let mut cmd: Option<String> = None;
    let mut args: Vec<String> = Vec::new();
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut cmd)
          .add_argument("command", box StoreOption::<String>,
                "A vagga command to run");
        ap.refer(&mut args)
          .add_argument("args", box List::<String>,
                "Arguments for the command");
        env_options(&mut env, &mut ap);
        ap.stop_on_first_argument(true);
        match ap.parse_args() {
            Ok(()) => {}
            Err(0) => return 0,
            Err(_) => return 122,
        }
    }

    if cmd.is_none() {
        err.write_line("Available commands:").ok();
        for (k, _) in env.config.commands.iter() {
            err.write_str("    ").ok();
            err.write_line(k.as_slice()).ok();
        }
        return 127;
    }

    let cmd = cmd.unwrap();
    args.insert(0, "vagga ".to_string() + cmd);
    let result = match cmd.as_slice() {
        "_build" => build_command(&mut env, args),
        "_run" => run_command(&mut env, args),
        _ => {
            // TODO(tailhook) look for commands in config
            match env.config.commands.find(&cmd) {
                Some(_) => run_user_command(&mut env, &cmd, args),
                None => {
                    err.write_line(
                        format!("Unknown command {}", cmd).as_slice()).ok();
                    return 127;
                }
            }
        }
    };
    match result {
        Ok(rc) => {
            return rc;
        }
        Err(text) =>  {
            err.write_line(text.as_slice()).ok();
            return 121;
        }
    }
}
