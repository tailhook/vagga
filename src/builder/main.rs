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

use std::os::{set_exit_status};
use std::io::pipe::PipeStream;
use libc::funcs::posix88::unistd::dup2;

use config::read_config;
use argparse::{ArgumentParser, Store};
use self::context::{BuildContext};
use self::bld::{BuildCommand};

mod context;
mod bld;
mod download;
mod tarcmd;
mod commands {
    pub mod debian;
}


pub fn run() -> int {
    let mut container: String = "".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            A tool which versions containers
            ");
        ap.refer(&mut container)
          .add_argument("container", box Store::<String>,
                "A container to version")
          .required();
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
    let mut build_context = BuildContext {
        container_name: container,
        container_config: (*cont).clone(),
    };
    debug!("Versioning items: {}", cont.setup.len());
    for b in cont.setup.iter() {
        debug!("Versioning setup: {}", b);
        match b.build(&mut build_context) {
            Ok(()) => {}
            Err(e) => {
                error!("Error versioning command {}: {}", b, e);
                return 1;
            }
        }
    }
    return 0;
}

fn main() {
    let val = run();
    set_exit_status(val);
}
