#![recursion_limit="100"]
#![cfg_attr(not(feature="containers"), allow(unused_imports))]
// TODO(tailhook) fix this one when migration is complete
#![cfg_attr(not(feature="containers"), allow(dead_code))]

use std::env;
use std::process::exit;

#[macro_use] extern crate failure;
#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;

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
