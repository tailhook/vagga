use std::old_io::stderr;
use std::os::{setenv, unsetenv, getenv, getcwd};
use std::env::{set_exit_status};
use config::find_config;
use container::signal;
use argparse::{ArgumentParser, Store, List, Collect};

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

    let workdir = getcwd().unwrap();

    let (config, cfg_dir) = match find_config(&workdir) {
        Ok(tup) => tup,
        Err(e) => {
            err.write_line(e.as_slice()).ok();
            return 126;
        }
    };
    let int_workdir = workdir.path_relative_from(&cfg_dir)
                             .unwrap_or(Path::new("."));

    for k in propagate_env.into_iter() {
        setenv(("VAGGAENV_".to_string() + &k[..]).as_slice(),
            getenv(k.as_slice()).unwrap_or("".to_string()));
    }
    for pair in set_env.into_iter() {
        let mut pairiter = pair.as_slice().splitn(1, '=');
        let key = "VAGGAENV_".to_string() + pairiter.next().unwrap();
        if let Some(value) = pairiter.next() {
            setenv(key.as_slice(), value.to_string());
        } else {
            unsetenv(key.as_slice());
        }
    }

    let result:Result<i32, String> = match cname.as_slice() {
        "" => {
            err.write_line("Available commands:").ok();
            for (k, cmd) in config.commands.iter() {
                err.write_str("    ").ok();
                err.write_str(k.as_slice()).ok();
                match cmd.description() {
                    Some(ref val) => {
                        if k.len() > 19 {
                            err.write_str("\n                        ").ok();
                        } else {
                            for _ in range(k.len(), 19) {
                                err.write_char(' ').ok();
                            }
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
            user::run_wrapper(Some(&int_workdir), cname, args, true)
        }
        "_build" => {
            build::build_command(&config, args)
        }
        "_run" => {
            underscore::run_command(&config, &int_workdir, cname, args)
        }
        "_run_in_netns" => {
            underscore::run_in_netns(&config, &int_workdir, cname, args)
        }
        _ => {
            user::run_user_command(&config, &int_workdir, cname, args)
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

fn main() {
    signal::block_all();
    let val = run();
    set_exit_status(val);
}
