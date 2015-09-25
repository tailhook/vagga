use std::env;
use std::io::{stderr, Write};
use std::path::{Path};
use std::process::exit;

use options;
use config::find_config;
use argparse::{ArgumentParser, Store, List, Collect};
use super::path_util::ToRelative;

mod list;
mod user;
mod network;
mod supervisor;
mod underscore;
mod build;


pub fn run() -> i32 {
    let mut err = stderr();
    let mut cname = "".to_string();
    let mut args = vec!();
    let mut set_env = Vec::<String>::new();
    let mut propagate_env = Vec::<String>::new();
    let mut build_mode = Default::default();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Runs a command in container, optionally builds container if that
            does not exists or outdated.

            Run `vagga` without arguments to see the list of commands.
            ");
        ap.refer(&mut set_env)
          .add_option(&["-E", "--env", "--environ"], Collect,
                "Set environment variable for running command")
          .metavar("NAME=VALUE");
        ap.refer(&mut propagate_env)
          .add_option(&["-e", "--use-env"], Collect,
                "Propagate variable VAR into command environment")
          .metavar("VAR");
        options::build_mode(&mut ap, &mut build_mode);
        ap.refer(&mut cname)
          .add_argument("command", Store,
                "A vagga command to run");
        ap.refer(&mut args)
          .add_argument("args", List,
                "Arguments for the command");
        ap.stop_on_first_argument(true);
        match ap.parse_args() {
            Ok(()) => {}
            Err(0) => return 0,
            Err(_) => return 122,
        }
    }

    let workdir = env::current_dir().unwrap();

    let (config, cfg_dir) = match find_config(&workdir) {
        Ok(tup) => tup,
        Err(e) => {
            writeln!(&mut err, "{}", e).ok();
            return 126;
        }
    };
    let int_workdir = workdir.rel_to(&cfg_dir)
                             .unwrap_or(&Path::new("."));

    for k in propagate_env.into_iter() {
        env::set_var(&("VAGGAENV_".to_string() + &k[..]),
            env::var(&k).unwrap_or("".to_string()));
    }
    for pair in set_env.into_iter() {
        let mut pairiter = pair[..].splitn(2, '=');
        let key = "VAGGAENV_".to_string() + pairiter.next().unwrap();
        if let Some(value) = pairiter.next() {
            env::set_var(&key, value.to_string());
        } else {
            env::remove_var(&key);
        }
    }

    let result:Result<i32, String> = match &cname[..] {
        "" => {
            writeln!(&mut err, "Available commands:").ok();
            for (k, cmd) in config.commands.iter() {
                write!(&mut err, "    {}", k).ok();
                match cmd.description() {
                    Some(ref val) => {
                        if k.len() > 19 {
                            write!(&mut err, "\n                        ").ok();
                        } else {
                            for _ in k.len()..19 {
                                err.write_all(b" ").ok();
                            }
                            err.write_all(b" ").ok();
                        }
                        err.write_all(val[..].as_bytes()).ok();
                    }
                    None => {}
                }
                err.write_all(b"\n").ok();
            }
            return 127;
        }
        "_create_netns" => {
            network::create_netns(&config, args)
        }
        "_destroy_netns" => {
            network::destroy_netns(&config, args)
        }
        "_list" => {
            list::print_list(&config, args)
        }
        "_build_shell" | "_clean" | "_version_hash" => {
            user::run_wrapper(Some(&int_workdir), cname, args, true, None)
        }
        "_build" => {
            build::build_command(&config, args)
        }
        "_run" => {
            underscore::run_command(&config, &int_workdir, cname, args,
                build_mode)
        }
        "_run_in_netns" => {
            underscore::run_in_netns(&config, &int_workdir, cname, args,
                build_mode)
        }
        _ => {
            user::run_user_command(&config, &int_workdir, cname, args,
                build_mode)
        }
    };

    match result {
        Ok(rc) => {
            return rc;
        }
        Err(text) =>  {
            writeln!(&mut err, "{}", text).ok();
            return 121;
        }
    }
}

pub fn main() {
    let val = run();
    exit(val);
}
