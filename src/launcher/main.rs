#![feature(phase)]

extern crate quire;
extern crate argparse;
extern crate serialize;
extern crate regex;
#[phase(plugin)] extern crate regex_macros;

extern crate config;

use std::io::stderr;
use std::os::{getcwd, set_exit_status};
use config::find_config;
use settings::read_settings;
use argparse::{ArgumentParser, StoreOption, List};

mod settings;


pub fn run() -> int {
    let mut err = stderr();
    let mut cmd: Option<String> = None;
    let mut args: Vec<String> = Vec::new();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Runs a command in container, optionally builds container if that
            does not exists or outdated.

            Run `vagga` without arguments to see the list of commands.
            ");
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

    let workdir = getcwd();

    let (config, project_root) = match find_config(&workdir) {
        Ok(tup) => tup,
        Err(e) => {
            err.write_line(e.as_slice()).ok();
            return 126;
        }
    };
    let (ext_settings, int_settings) = match read_settings(&project_root) {
        Ok(tup) => tup,
        Err(e) => {
            err.write_line(e.as_slice()).ok();
            return 126;
        }
    };


    if cmd.is_none() {
        err.write_line("Available commands:").ok();
        for (k, cmd) in config.commands.iter() {
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

    //match Ok(0) {
    //    Ok(rc) => {
    //        return rc;
    //    }
    //    Err(text) =>  {
    //        err.write_line(text.as_slice()).ok();
    //        return 121;
    //    }
    //}
    return 0;
}

fn main() {
    let val = run();
    set_exit_status(val);
}
