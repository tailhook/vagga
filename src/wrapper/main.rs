#![feature(phase, if_let, slicing_syntax)]

extern crate quire;
extern crate argparse;
extern crate serialize;
extern crate regex;
extern crate libc;
#[phase(plugin)] extern crate regex_macros;
#[phase(plugin, link)] extern crate log;

extern crate config;
#[phase(plugin, link)] extern crate container;

use std::io::stderr;
use std::os::{getcwd, set_exit_status};

use config::{find_config, Config, Settings};
use config::command::main::{Command, Supervise};
use container::signal;
use settings::{read_settings, MergedSettings};
use argparse::{ArgumentParser, Store, List};

mod settings;
mod debug;
mod build;
mod run;
mod supervise;
mod commandline;
mod setup;
mod util;


struct Wrapper<'a> {
    config: &'a Config,
    settings: &'a Settings,
    project_root: &'a Path,
    ext_settings: &'a MergedSettings,
}


pub fn run() -> int {
    let mut err = stderr();
    let mut cmd: String = "".to_string();
    let mut args: Vec<String> = Vec::new();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Internal vagga tool to setup basic system sandbox
            ");
        ap.refer(&mut cmd)
          .add_argument("command", box Store::<String>,
                "A vagga command to run")
          .required();
        ap.refer(&mut args)
          .add_argument("args", box List::<String>,
                "Arguments for the command");
        ap.stop_on_first_argument(true);
        match ap.parse_args() {
            Ok(()) => {}
            Err(0) => return 0,
            Err(_) => {
                return 122;
            }
        }
    }

    let workdir = getcwd();

    let (config, project_root) = match find_config(&workdir) {
        Ok(tup) => tup,
        Err(e) => {
            err.write_line(e.as_slice()).ok();
            return 126;
        }
    };
    let (ext_settings, int_settings) = match read_settings(&project_root)
    {
        Ok(tup) => tup,
        Err(e) => {
            err.write_line(e.as_slice()).ok();
            return 126;
        }
    };

    let wrapper = Wrapper {
        config: &config,
        settings: &int_settings,
        project_root: &project_root,
        ext_settings: &ext_settings,
    };

    args.insert(0, format!("vagga {}", cmd));

    let result = match cmd.as_slice() {
        "_build_shell" => Ok(debug::run_interactive_build_shell(&wrapper)),
        "_build" => build::build_container_cmd(&wrapper, args),
        "_version_hash" => build::print_version_hash_cmd(&wrapper, args),
        "_run" => run::run_command_cmd(&wrapper, args, true),
        "_run_in_netns" => run::run_command_cmd(&wrapper, args, false),
        _ => {
            match config.commands.find(&cmd) {
                Some(&Command(ref cmd_info)) => {
                    commandline::commandline_cmd(cmd_info, &wrapper, args)
                }
                Some(&Supervise(ref svc_info)) => {
                    supervise::supervise_cmd(svc_info, &wrapper, args)
                }
                None => {
                    error!("Unknown command {}", cmd);
                    return 127;
                }
            }
        }
    };
    match result {
        Ok(x) => return x,
        Err(e) => {
            error!("Error executing {}: {}", cmd, e);
            return 124;
        }
    };
}

fn main() {
    signal::block_all();
    let val = run();
    set_exit_status(val);
}
