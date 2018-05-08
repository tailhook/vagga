#![recursion_limit="100"]

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
extern crate signal;
extern crate regex;
extern crate scan_dir;
extern crate zip;
extern crate tar;
extern crate flate2;
extern crate xz2;
extern crate bzip2;
extern crate net2;
extern crate docopt;
extern crate humantime;
extern crate digest_writer;
extern crate itertools;
extern crate git2;
extern crate path_filter;
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

#[cfg(feature="containers")] extern crate unshare;
#[cfg(feature="containers")] extern crate libmount;
#[cfg(feature="containers")] extern crate dir_signature;

#[cfg(feature="containers")] #[macro_use] mod macros;
#[cfg(feature="containers")] mod config;
#[cfg(feature="containers")] mod container;
#[cfg(feature="containers")] mod file_util;
#[cfg(feature="containers")] mod path_util;
#[cfg(feature="containers")] mod process_util;
#[cfg(feature="containers")] mod tty_util;
#[cfg(feature="containers")] mod options;
#[cfg(feature="containers")] mod digest;
#[cfg(feature="containers")] mod build_step;
#[cfg(feature="containers")] mod storage_dir;

// Commands
#[cfg(feature="containers")] mod launcher;
#[cfg(feature="containers")] mod network;
#[cfg(feature="containers")] mod setup_netns;
#[cfg(feature="containers")] mod version;
#[cfg(feature="containers")] mod wrapper;
#[cfg(feature="containers")] mod builder;
#[cfg(feature="containers")] mod runner;
#[cfg(feature="containers")] mod capsule;

fn init_logging() {
    if let Err(_) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "warn");
    }
    env_logger::init();
}

#[cfg(feature="containers")]
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
        "launcher" => launcher::run(args),
        "network" => network::run(args),
        "setup_netns" => setup_netns::run(args),
        "version" => version::run(args),
        "wrapper" => wrapper::run(args),
        "build" | "builder" => builder::run(args),
        "runner" => runner::run(args),
        _ => launcher::run(args),
    };
    exit(code);
}

#[cfg(all(not(feature="containers"), not(feature="docker_runner")))]
fn main() {
    unimplemented!();
}

#[cfg(feature="docker_runner")]
fn main() {
    init_logging();
    exit(launcher::run(env::args().collect()));
}
