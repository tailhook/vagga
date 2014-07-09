#![feature(phase)]

extern crate serialize;
extern crate collections;
extern crate debug;
extern crate libc;
extern crate green;
extern crate rustuv;
#[phase(plugin, link)] extern crate log;

extern crate quire;
extern crate argparse;


use std::os::set_exit_status;

use self::main::run;


mod config;
mod build;
mod run;
mod env;
mod main;


fn main() {
    set_exit_status(run());
}
