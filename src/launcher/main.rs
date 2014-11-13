#![feature(phase, if_let)]

extern crate quire;
extern crate argparse;
extern crate serialize;
extern crate regex;
#[phase(plugin)] extern crate regex_macros;

extern crate config;
extern crate container;

use std::rc::Rc;
use std::cell::Cell;
use std::io::stderr;
use std::time::duration::Duration;
use std::os::{getcwd, getenv, set_exit_status, self_exe_path};
use config::find_config;
use container::signal;
use container::monitor::{Monitor, Executor};
use container::container::{Command};
use argparse::{ArgumentParser, Store, StoreOption, List};

mod list;


struct RunWrapper {
    cmd: String,
    args: Vec<String>,
    extra_settings: Option<Path>,
    result: Cell<int>,
}


impl Executor for RunWrapper {
    fn command(&self) -> Command {
        let mut cmd = Command::new("wrapper".to_string(),
            self_exe_path().unwrap().join("wrapper"));
        cmd.keep_sigmask();
        if let Some(ref ex) = self.extra_settings {
            cmd.arg("--extra-settings");
            cmd.arg(ex);
        }
        cmd.arg(self.cmd.as_slice());
        cmd.args(self.args.as_slice());
        cmd.set_env("TERM".to_string(),
                    getenv("TERM").unwrap_or("dumb".to_string()));
        if let Some(x) = getenv("RUST_LOG") {
            cmd.set_env("RUST_LOG".to_string(), x);
        }
        if let Some(x) = getenv("RUST_BACKTRACE") {
            cmd.set_env("RUST_BACKTRACE".to_string(), x);
        }
        cmd.container();
        cmd.set_max_uidmap();
        return cmd;
    }
    fn finish(&self, status: int) -> bool {
        self.result.set(status);
        return false;
    }
}


pub fn run() -> int {
    let mut err = stderr();
    let result = Cell::new(-1);
    let mut wrapper = RunWrapper {
        cmd: "".to_string(),
        args: vec!(),
        extra_settings: None,
        result: result,
    };
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Runs a command in container, optionally builds container if that
            does not exists or outdated.

            Run `vagga` without arguments to see the list of commands.
            ");
        ap.refer(&mut wrapper.extra_settings)
          .add_option(["--extra-settings"], box StoreOption::<Path>,
                "Extra settings file to use (mostly for tests)");
        ap.refer(&mut wrapper.cmd)
          .add_argument("command", box Store::<String>,
                "A vagga command to run");
        ap.refer(&mut wrapper.args)
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

    let result:Result<int, String> = match wrapper.cmd.as_slice() {
        "" => {
            err.write_line("Available commands:").ok();
            for (k, cmd) in config.commands.iter() {
                err.write_str("    ").ok();
                err.write_str(k.as_slice()).ok();
                match cmd.description {
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
            list::print_list(&config, wrapper.args)
        }
        _ => {
            let mut mon = Monitor::new("launcher".to_string());
            mon.add(Rc::new("wrapper".to_string()), box wrapper,
                Duration::seconds(0), None);
            mon.run();
            Ok(result.get())
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
