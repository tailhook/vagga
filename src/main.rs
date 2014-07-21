use argparse::{ArgumentParser, StoreOption, List};
use std::os::{getcwd, args, self_exe_path, self_exe_name};
use std::io::stdio::stderr;

use super::config::find_config;
use super::config::{Shell, Plain, Supervise};
use super::build::build_command;
use super::run::run_command_line;
use super::commands::shell::run_shell_command;
use super::commands::command::run_plain_command;
use super::commands::supervise::run_supervise_command;
use super::env::Environ;
use super::options::env_options;
use super::settings::{Settings, read_settings, set_variant};


pub fn run() -> int {
    let mut err = stderr();
    let workdir = getcwd();

    let (config, project_root) = match find_config(&workdir) {
        Ok(tup) => tup,
        Err(e) => {
            err.write_line(e.as_slice()).ok();
            return 126;
        }
    };
    let mut env = Environ {
        vagga_path: self_exe_path().unwrap(),
        vagga_exe: self_exe_name().unwrap(),
        work_dir: workdir,
        local_vagga: project_root.join(".vagga"),
        project_root: project_root,
        variables: Vec::new(),
        config: config,
        settings: Settings::new(),
    };
    read_settings(&mut env);

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
        for (k, cmd) in env.config.commands.iter() {
            err.write_str("    ").ok();
            err.write_str(k.as_slice()).ok();
            match cmd.description {
                Some(ref val) => {
                    for _ in range(k.len(), 20) {
                        err.write_char(' ').ok();
                    }
                    err.write_str(val.as_slice()).ok();
                }
                None => {}
            }
            err.write_char('\n').ok();
        }
        return 127;
    }

    let cname = cmd.unwrap();
    args.insert(0, "vagga ".to_string() + cname);
    let result = match cname.as_slice() {
        "_build" => build_command(&mut env, args),
        "_run" => run_command_line(&mut env, args),
        "_setv" | "_setvariant" => set_variant(&mut env, args),
        _ => {
            let fun = match env.config.commands.find(&cname) {
                Some(ref cmd) => {
                    match cmd.execute {
                        Shell(_) => run_shell_command,
                        Plain(_) => run_plain_command,
                        Supervise(_, _) => run_supervise_command,
                    }
                }
                None => {
                    err.write_line(
                        format!("Unknown command {}", cname).as_slice()).ok();
                    return 127;
                }
            };
            fun(&mut env, &cname, args)
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
