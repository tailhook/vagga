use argparse::{ArgumentParser, StoreOption, List};
use std::os::{getcwd, args};
use std::io::stdio::stderr;

use super::config::find_config;
use super::build::build_command;
use super::run::run_command;
use super::env::Environ;

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
    let env = Environ {
        vagga_dir: mypath.dir_path(),
        vagga_path: mypath,
        vagga_command: vcmd.clone(),
        work_dir: workdir,
        project_root: project_root,
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
        ap.stop_on_first_argument(true);
        match ap.parse_args() {
            Ok(()) => {}
            Err(0) => return 0,
            Err(_) => return 122,
        }
    }

    if cmd.is_none() {
        err.write_line("Available commands:").ok();
        for (k, _) in config.commands.iter() {
            err.write_str("    ").ok();
            err.write_line(k.as_slice()).ok();
        }
        return 127;
    }

    let cmd = cmd.unwrap();
    args.insert(0, "vagga ".to_string() + cmd);
    let result = match cmd.as_slice() {
        "_build" => build_command(&env, &config, args),
        "_run" => run_command(&env, &config, args),
        x => {
            // TODO(tailhook) look for commands in config
            err.write_line(format!("Unknown command {}", x).as_slice()).ok();
            return 127;  // Like shell exit code
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
