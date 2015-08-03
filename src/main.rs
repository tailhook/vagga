#![feature(path_ext)]
use std::env;

extern crate shaman;
extern crate libc;
extern crate nix;
extern crate rand;
extern crate rustc_serialize;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate argparse;
extern crate quire;
extern crate time;

#[macro_use] mod macros;
mod config;
mod container;
mod file_util;
mod path_util;

// Commands
mod launcher;
mod network;
mod setup_netns;
mod version;
mod wrapper;
mod builder;

fn main() {
    env_logger::init().unwrap();
    match env::args().next().as_ref().map(|x| &x[..]) {
        Some("vagga") => launcher::main(),
        Some("vagga_launcher") => launcher::main(),
        Some("vagga_network") => network::main(),
        Some("vagga_setup_netns") => setup_netns::main(),
        Some("vagga_version") => version::main(),
        Some("vagga_wrapper") => wrapper::main(),
        Some("vagga_build") => builder::main(),
        _ => launcher::main(),
    }
}
