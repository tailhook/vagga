extern crate shaman;
extern crate rustc_serialize;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate argparse;

mod builder;
mod config;
mod container;
mod launcher;
mod network;
mod setup_netns;
mod version;
mod wrapper;

fn main() {
    env_logger::init().unwrap();
}
