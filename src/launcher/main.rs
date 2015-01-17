#![feature(phase, if_let, slicing_syntax)]

extern crate quire;
extern crate argparse;
extern crate serialize;
extern crate libc;
extern crate regex;
#[phase(plugin)] extern crate regex_macros;
#[phase(plugin, link)] extern crate log;

extern crate config;
extern crate container;

use std::io::stderr;
use std::os::{getcwd, set_exit_status};
use config::find_config;
use container::signal;
use argparse::{ArgumentParser, Store, List};

mod list;
mod user;
mod network;
mod supervisor;


pub fn run() -> int {
    let mut err = stderr();
    let mut cname = "".to_string();
    let mut args = vec!();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Runs a command in container, optionally builds container if that
            does not exists or outdated.

            Run `vagga` without arguments to see the list of commands.
            ");
        ap.refer(&mut cname)
          .add_argument("command", box Store::<String>,
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

    let workdir = getcwd();

    let (config, cfg_dir) = match find_config(&workdir) {
        Ok(tup) => tup,
        Err(e) => {
            err.write_line(e.as_slice()).ok();
            return 126;
        }
    };
    let int_workdir = workdir.path_relative_from(&cfg_dir)
                             .unwrap_or(Path::new("."));

    let result:Result<int, String> = match cname.as_slice() {
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
        "_run" | "_build" | "_build_shell" => {
            user::run_wrapper(&int_workdir, cname, args, true)
        }
        "_run_in_netns" => {
            network::run_in_netns(&int_workdir, cname, args)
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
