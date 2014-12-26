#![feature(phase, if_let, slicing_syntax, macro_rules)]

extern crate quire;
extern crate argparse;
extern crate serialize;
extern crate regex;
extern crate libc;
#[phase(plugin)] extern crate regex_macros;
#[phase(plugin, link)] extern crate log;

extern crate config;
#[phase(plugin, link)] extern crate container;

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
mod commands {
    pub mod debian;
    pub mod generic;
    pub mod alpine;
}


pub fn run() -> int {
    signal::block_all();
    let mut container: String = "".to_string();
    let mut settings: Settings = Default::default();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            A tool which versions containers
            ");
        ap.refer(&mut container)
          .add_argument("container", box Store::<String>,
                "A container to version")
          .required();
        ap.refer(&mut settings)
          .add_option(&["--settings"], box Store::<Settings>,
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
    let cont = cfg.containers.find(&container)
        .expect("Container not found");  // TODO
    let mut build_context = BuildContext::new(container, (*cont).clone());
    match build_context.start() {
        Ok(()) => {}
        Err(e) => {
            error!("Error preparing for build: {}", e);
            return 1;
        }
    }
    for b in cont.setup.iter() {
        debug!("Versioning setup: {}", b);
        match b.build(&mut build_context) {
            Ok(()) => {}
            Err(e) => {
                error!("Error build command {}: {}", b, e);
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
