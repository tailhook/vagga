#![feature(phase, if_let)]

extern crate quire;
extern crate argparse;
extern crate serialize;
extern crate regex;
#[phase(plugin)] extern crate regex_macros;

extern crate config;
#[phase(plugin, link)] extern crate container;

use std::cell::Cell;
use std::io::stderr;
use std::io::{TypeSymlink, TypeDirectory, PathDoesntExist};
use std::os::{getcwd, getenv, set_exit_status, self_exe_path};
use std::io::FilePermission;
use std::io::fs::{mkdir, walk_dir, copy};
use std::io::fs::PathExtensions;
use std::io::fs::File;
use config::find_config;
use container::signal;
use container::monitor::{Monitor, Executor};
use container::container::{Command};
use container::mount::{mount_tmpfs, bind_mount, mount_private, unmount};
use container::mount::{mount_ro_recursive, mount_pseudo};
use container::mount::{get_submounts_of};
use container::root::change_root;
use settings::read_settings;
use argparse::{ArgumentParser, Store, StoreOption, List};

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


pub fn setup_filesystem(project_root: &Path) -> Result<(), String> {
    let mnt_dir = project_root.join(".vagga/.mnt");
    try!(make_mountpoint(project_root));
    try!(mount_tmpfs(&mnt_dir, "size=10m"));
    let vagga_dir = mnt_dir.join("vagga");

    let proc_dir = mnt_dir.join("proc");
    try_str!(mkdir(&proc_dir,
        FilePermission::from_bits_truncate(0o755)));
    try!(mount_pseudo(&proc_dir, "proc", "", true));

    let vagga_dir = mnt_dir.join("vagga");
    try_str!(mkdir(&vagga_dir,
        FilePermission::from_bits_truncate(0o755)));

    let bin_dir = vagga_dir.join("bin");
    try_str!(mkdir(&bin_dir,
        FilePermission::from_bits_truncate(0o755)));
    try!(bind_mount(&self_exe_path().unwrap(), &bin_dir));
    try!(mount_ro_recursive(&bin_dir));

    let etc_dir = vagga_dir.join("etc");
    try_str!(mkdir(&etc_dir,
        FilePermission::from_bits_truncate(0o755)));
    try!(copy(&Path::new("/etc/hosts"), &etc_dir.join("hosts"))
        .map_err(|e| format!("Error copying /etc/hosts: {}", e)));
    try!(copy(&Path::new("/etc/resolv.conf"), &etc_dir.join("resolv.conf"))
        .map_err(|e| format!("Error copying /etc/resolv.conf: {}", e)));

    let old_root = vagga_dir.join("old_root");
    try_str!(mkdir(&old_root,
        FilePermission::from_bits_truncate(0o755)));
    try!(change_root(&mnt_dir, &old_root));
    try!(unmount(&Path::new("/vagga/old_root")));

    return Ok(());
}


pub fn run() -> int {
    let mut err = stderr();
    let mut cmd: String = "".to_string();
    let mut extra_settings: Option<Path> = None;
    let mut args: Vec<String> = Vec::new();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Runs a command in container, optionally builds container if that
            does not exists or outdated.

            Run `vagga` without arguments to see the list of commands.
            ");
        ap.refer(&mut extra_settings)
          .add_option(["--extra-settings"], box StoreOption::<Path>,
                "Extra settings file to use (mostly for tests)");
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
    let (ext_settings, int_settings) = match read_settings(
        &project_root, &extra_settings)
    {
        Ok(tup) => tup,
        Err(e) => {
            err.write_line(e.as_slice()).ok();
            return 126;
        }
    };

    match setup_filesystem(&project_root) {
        Ok(()) => {
            return 0;
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
