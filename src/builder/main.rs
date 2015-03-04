#![feature(slicing_syntax)]

extern crate quire;
extern crate argparse;
extern crate serialize;
extern crate libc;
#[macro_use] extern crate log;

extern crate config;
#[macro_use] extern crate container;

use std::default::Default;
use std::env::{set_exit_status};

use config::read_config;
use config::Settings;
use container::signal;
use argparse::{ArgumentParser, Store};
use self::context::{BuildContext};
use self::bld::{BuildCommand};

mod context;
mod bld;
mod download;
mod tarcmd;
mod commands {
    pub mod debian;
    pub mod generic;
    pub mod alpine;
    pub mod pip;
    pub mod npm;
}
mod capsule;
mod packages;
mod timer;


pub fn run() -> i32 {
    signal::block_all();
    let mut container: String = "".to_string();
    let mut settings: Settings = Default::default();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            A tool which versions containers
           -lang.org/ ");
        ap.refer(&mut container)
          .add_argument("container", Store,
                "A container to version")
          .required();
        ap.refer(&mut settings)
          .add_option(&["--settings"], Store,
                "User settings for the container build");
        match ap.parse_args() {
            Ok(()) => {}
            Err(0) => return 0,
            Err(_) => return 122,
        }
    }

    _build(&container, settings)
        .map(|()| 0)
        .map_err(|e| error!("Error building container {:?}: {}", container, e))
        .unwrap_or(1)
}

fn _build(container: &String, settings: Settings) -> Result<(), String> {
    // TODO(tailhook) read also config from /work/.vagga/vagga.yaml
    let cfg = read_config(&Path::new("/work/vagga.yaml")).ok()
        .expect("Error parsing configuration file");  // TODO
    let cont = cfg.containers.get(container)
        .expect("Container not found");  // TODO

    let mut build_context = BuildContext::new(
        &cfg, container.clone(), cont, settings);
    try!(build_context.start());

    for b in cont.setup.iter() {
        debug!("Building step: {:?}", b);
        try!(b.build(&mut build_context, true));
    }

    try!(build_context.finish());
    Ok(())
}

fn main() {
    let val = run();
    set_exit_status(val);
}
