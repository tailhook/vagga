#![feature(phase, if_let)]

extern crate quire;
extern crate argparse;
extern crate serialize;
extern crate regex;
#[phase(plugin)] extern crate regex_macros;

extern crate config;
extern crate container;

use std::cell::Cell;
use std::io::stderr;
use std::io::{TypeSymlink, TypeDirectory, PathDoesntExist};
use std::os::{getcwd, getenv, set_exit_status, self_exe_path};
use std::io::FilePermission;
use std::io::fs::mkdir;
use std::io::fs::PathExtensions;
use config::find_config;
use container::signal;
use container::monitor::{Monitor, Executor};
use container::container::{Command};
use settings::read_settings;
use argparse::{ArgumentParser, Store, List};

mod settings;


pub fn make_mountpoint(project_root: &Path) -> Result<(), String> {
    let vagga_dir = project_root.join(".vagga");
    match vagga_dir.lstat() {
        Ok(stat) if stat.kind == TypeSymlink => {
            return Err(concat!("The `.vagga` dir can't be a symlink. ",
                               "Please run `unlink .vagga`").to_string());
        }
        Ok(stat) if stat.kind == TypeDirectory => {
            // ok
        }
        Ok(_) => {
            return Err(concat!("The `.vagga` must be a directory. ",
                               "Please run `unlink .vagga`").to_string());
        }
        Err(ref e) if e.kind == PathDoesntExist => {
            try!(mkdir(&vagga_dir,
                FilePermission::from_bits_truncate(0o755))
                .map_err(|e| format!("Can't create {}: {}",
                                     vagga_dir.display(), e)));
        }
        Err(ref e) => {
            return Err(format!("Can't stat `.vagga`: {}", e));
        }
    }
    let mnt_dir = vagga_dir.join(".mnt");
    match mnt_dir.lstat() {
        Ok(stat) if stat.kind == TypeSymlink => {
            return Err(concat!("The `.vagga/.mnt` dir can't be a symlink. ",
                               "Please run `unlink .vagga/.mnt`").to_string());
        }
        Ok(stat) if stat.kind == TypeDirectory => {
            // ok
        }
        Ok(_) => {
            return Err(concat!("The `.vagga/.mnt` must be a directory. ",
                               "Please run `unlink .vagga/.mnt`").to_string());
        }
        Err(ref e) if e.kind == PathDoesntExist => {
            try!(mkdir(&mnt_dir,
                FilePermission::from_bits_truncate(0o755))
                .map_err(|e| format!("Can't create {}: {}",
                                     mnt_dir.display(), e)));
        }
        Err(ref e) => {
            return Err(format!("Can't stat `.vagga/.mnt`: {}", e));
        }
    }
    return Ok(());
}


pub fn run() -> int {
    let mut err = stderr();
    let mut cmd: String = "".to_string();
    let mut args: Vec<String> = Vec::new();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Runs a command in container, optionally builds container if that
            does not exists or outdated.

            Run `vagga` without arguments to see the list of commands.
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



    let result = make_mountpoint(&project_root).map(|_| 0);

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
