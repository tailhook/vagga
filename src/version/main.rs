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
use container::signal;
use argparse::{ArgumentParser, Store};
use container::sha256::{Sha256, Digest};
use self::version::{VersionHash, Hashed, New, Error};

mod version;


pub fn run() -> int {
    signal::block_all();
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
    debug!("Versioning items: {}", cont.setup.len());
    let mut hash = Sha256::new();
    for b in cont.setup.iter() {
        debug!("Versioning setup: {}", b);
        match b.hash(&mut hash) {
            Hashed => continue,
            New => return 29,  // Always rebuild
            Error(e) => {
                error!("Error versioning command {}: {}", b, e);
                return 1;
            }
        }
    }
    match PipeStream::open(3).write_str(hash.result_str().as_slice()) {
        Ok(()) => {}
        Err(e) => {
            error!("Error writing hash: {}", e);
            return 1;
        }
    }
    return 0;
}

fn main() {
    // let's make stdout safer
    unsafe { dup2(1, 3) };
    unsafe { dup2(2, 1) };

    let val = run();
    set_exit_status(val);
}
