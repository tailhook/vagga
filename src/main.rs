#![recursion_limit="100"]
#![cfg_attr(not(feature="containers"), allow(unused_imports))]
// TODO(tailhook) fix this one when migration is complete
#![cfg_attr(not(feature="containers"), allow(dead_code))]

use std::env;
use std::process::exit;

extern crate sha2;
extern crate blake2;
extern crate typenum;
extern crate libc;
extern crate nix;
extern crate rand;
extern crate env_logger;
extern crate argparse;
extern crate quire;
extern crate regex;
extern crate scan_dir;
extern crate docopt;
extern crate humantime;
extern crate digest_writer;
extern crate itertools;
extern crate serde;
extern crate serde_json;
extern crate resolv_conf;
#[macro_use] extern crate failure;
#[macro_use] extern crate matches;
#[macro_use] extern crate mopa;
#[macro_use] extern crate log;
#[macro_use] extern crate quick_error;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;

#[cfg(feature="containers")] extern crate tempfile;
#[cfg(feature="containers")] extern crate git2;
#[cfg(feature="containers")] extern crate bzip2;
#[cfg(feature="containers")] extern crate signal;
#[cfg(feature="containers")] extern crate tar;
#[cfg(feature="containers")] extern crate zip;
#[cfg(feature="containers")] extern crate xz2;
#[cfg(feature="containers")] extern crate net2;
#[cfg(feature="containers")] extern crate dir_signature;
#[cfg(feature="containers")] extern crate flate2;
#[cfg(feature="containers")] extern crate libmount;
#[cfg(feature="containers")] extern crate path_filter;
#[cfg(feature="containers")] extern crate unshare;

#[cfg(feature="containers")] #[macro_use] mod macros;
mod config;
#[cfg(feature="containers")] mod container;
#[cfg(feature="containers")] mod file_util;
mod path_util;
#[cfg(feature="containers")] mod process_util;
#[cfg(feature="containers")] mod tty_util;
mod options;
mod digest;
mod build_step;
#[cfg(feature="containers")] mod storage_dir;

// Commands
#[cfg(any(feature="containers", feature="config_runner"))]
mod launcher;
#[cfg(feature="containers")] mod network;
#[cfg(feature="containers")] mod setup_netns;
mod version;
#[cfg(feature="containers")] mod wrapper;
mod builder;
#[cfg(feature="containers")] mod runner;
#[cfg(feature="containers")] mod capsule;

fn init_logging() {
    if let Err(_) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "warn");
    }
    env_logger::init();
}

#[cfg(any(feature="containers", feature="config_runner"))]
fn main() {
    init_logging();
    let mut args = env::args().collect::<Vec<_>>();
    // TODO(tailhook) check if arg0 is "vagga" or "/proc/self/exe", maybe
    let cmd;
    let ep = if args.get(1).map(|x| x.starts_with("__") && x.ends_with("__"))
                .unwrap_or(false)
    {
        cmd = args.remove(1);
        cmd[2..cmd.len()-2].to_string()
    } else if args.get(0).map(|x| x.starts_with("vagga_")).unwrap_or(false) {
        args[0][6..].to_string()
    } else {
        "".to_string()
    };
    let code = match &ep[..] {
        #[cfg(feature="containers")]
        "launcher" => launcher::run(args),
        #[cfg(feature="containers")]
        "network" => network::run(args),
        #[cfg(feature="containers")]
        "setup_netns" => setup_netns::run(args),
        #[cfg(feature="containers")]
        "version" => version::run(args),
        #[cfg(feature="containers")]
        "wrapper" => wrapper::run(args),
        #[cfg(feature="containers")]
        "build" | "builder" => builder::run(args),
        #[cfg(feature="containers")]
        "runner" => runner::run(args),
        #[cfg(any(feature="containers", feature="config_runner"))]
        _ => launcher::run(args),
    };
    exit(code);
}

#[cfg(not(any(feature="containers", feature="config_runner")))]
fn main() {
    eprintln!("No runner configured. Use one of:");
    eprintln!("  --features=containers");
    eprintln!("  --features=config_runner");
}
