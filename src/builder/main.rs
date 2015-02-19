#![feature(slicing_syntax)]

extern crate quire;
extern crate argparse;
extern crate serialize;
extern crate regex;
extern crate libc;
#[macro_use] extern crate log;

extern crate config;
#[macro_use] extern crate container;

use std::default::Default;
use std::os::{set_exit_status};

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
mod dev;
mod commands {
    pub mod debian;
    pub mod generic;
    pub mod alpine;
    pub mod pip;
    pub mod npm;
}
mod capsule;


pub fn run() -> isize {
    signal::block_all();
    let mut container: String = "".to_string();
    let mut settings: Settings = Default::default();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            A tool which versions containers
            ");
        ap.refer(&mut container)
          .add_argument("container", Box::new(Store::<String>),
                "A container to version")
          .required();
        ap.refer(&mut settings)
          .add_option(&["--settings"], Box::new(Store::<Settings>),
                "User settings for the container build");
        match ap.parse_args() {
            Ok(()) => {}
            Err(0) => return 0,
            Err(_) => return 122,
        }
    }

    // TODO(tailhook) read also config from /work/.vagga/vagga.yaml
    let cfg = read_config(&Path::new("/work/vagga.yaml")).ok()
        .expect("Error parsing configuration file");  // TODO
    let cont = cfg.containers.get(&container)
        .expect("Container not found");  // TODO
    let mut build_context = BuildContext::new(
        &cfg, container, cont, settings);
    match build_context.start() {
        Ok(()) => {}
        Err(e) => {
            error!("Error preparing for build: {}", e);
            return 1;
        }
    }
    for b in cont.setup.iter() {
        debug!("Building step: {:?}", b);
        match b.configure(&mut build_context)
              .and_then(|()| b.build(&mut build_context))
        {
            Ok(()) => {}
            Err(e) => {
                error!("Error build command {:?}: {}", b, e);
                return 1;
            }
        }
    }

    match build_context.finish() {
        Ok(()) => {}
        Err(e) => {
            error!("Error finalizing container: {}", e);
            return 1;
        }
    }
    return 0;
}

fn main() {
    let val = run();
    set_exit_status(val);
}
