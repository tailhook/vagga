#![feature(phase)]

extern crate serialize;
extern crate collections;
extern crate debug;
extern crate libc;
extern crate green;
extern crate rustuv;
extern crate regex;
#[phase(plugin, link)] extern crate log;
#[phase(plugin)] extern crate regex_macros;

extern crate quire;
extern crate argparse;


use std::os::set_exit_status;

use self::main::run;


mod config;
mod build;
mod run;
mod env;
mod main;
mod linux;
mod options;


fn main() {
    set_exit_status(run());
}
