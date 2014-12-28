#![feature(phase, if_let)]

extern crate quire;
extern crate argparse;
extern crate serialize;
extern crate regex;
#[phase(plugin)] extern crate regex_macros;

extern crate config;
extern crate container;

use std::io::stderr;
use std::os::{getcwd, getenv, set_exit_status, self_exe_path};
use config::find_config;
use container::signal;
use container::monitor::{Monitor};
use container::monitor::{Killed, Exit};
use container::container::{Command};
use argparse::{ArgumentParser, Store, List};

mod list;


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
        "_list" => {
            list::print_list(&config, args)
        }
        _ => {
            let mut cmd = Command::new("wrapper".to_string(),
                self_exe_path().unwrap().join("vagga_wrapper"));
            cmd.keep_sigmask();
            cmd.arg(cname.as_slice());
            cmd.args(args.as_slice());
            cmd.set_env("TERM".to_string(),
                        getenv("TERM").unwrap_or("dumb".to_string()));
            if let Some(x) = getenv("RUST_LOG") {
                cmd.set_env("RUST_LOG".to_string(), x);
            }
            if let Some(x) = getenv("RUST_BACKTRACE") {
                cmd.set_env("RUST_BACKTRACE".to_string(), x);
            }
            if let Some(x) = getenv("HOME") {
                cmd.set_env("VAGGA_USER_HOME".to_string(), x);
            }
            cmd.set_env("PWD".to_string(), Path::new("/work")
                .join(workdir.path_relative_from(&cfg_dir)
                    .unwrap_or(Path::new(".")))
                .display().to_string());
            cmd.container();
            cmd.set_max_uidmap();
            match Monitor::run_command(cmd) {
                Killed => Ok(143),
                Exit(val) => Ok(val),
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

fn main() {
    signal::block_all();
    let val = run();
    set_exit_status(val);
}
