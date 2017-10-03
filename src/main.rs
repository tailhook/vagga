#![recursion_limit="100"]

use std::env;
use std::ffi::OsStr;
use std::io::{self, Write, stdout};
use std::path::PathBuf;
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
#[macro_use] extern crate matches;
#[macro_use] extern crate mopa;
#[macro_use] extern crate log;
#[macro_use] extern crate quick_error;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;

#[cfg(feature="containers")]
extern crate unshare;
#[cfg(feature="containers")]
extern crate libmount;
#[cfg(feature="containers")]
extern crate dir_signature;

use argparse::{ArgumentParser, StoreTrue};

#[macro_use] mod macros;
mod config;
mod container;
mod file_util;
mod path_util;
mod process_util;
mod tty_util;
mod options;
mod digest;
mod build_step;
mod storage_dir;

#[cfg(not(feature="containers"))]
mod unshare;
#[cfg(not(feature="containers"))]
mod libmount;
#[cfg(not(feature="containers"))]
mod dir_signature;

// Commands
mod launcher;
mod network;
mod setup_netns;
mod version;
mod wrapper;
mod builder;
mod runner;
mod capsule;

fn init_logging() {
    if let Err(_) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "warn");
    }
    env_logger::init().unwrap();
}

struct DevNull;

impl Write for DevNull {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(feature="containers")]
fn main() {
    init_logging();
    let mut args = env::args().collect::<Vec<_>>();
    let arg0 = args.get(0).map(|n| n.as_str()).unwrap_or("").to_string();
    let _arg0_path = PathBuf::from(&arg0);
    let exe_name = _arg0_path.file_name().unwrap_or(OsStr::new(""));
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

    if exe_name == "vagga" {
        let mut show_help = false;
        let mut show_version = false;
        let parse_res = {
            let cmdline = args.clone();
            let mut ap = ArgumentParser::new();
            ap.set_description("Show vagga version");
            ap.refer(&mut show_help)
                .add_option(&["-h", "--help"], StoreTrue,
                            "Show vagga help and exit");
            ap.refer(&mut show_version)
                .add_option(&["-V", "--version"], StoreTrue,
                            "Show vagga version and exit");
            ap.stop_on_first_argument(true);
            ap.silence_double_dash(false);
            ap.parse(cmdline, &mut DevNull {}, &mut DevNull {})
        };
        match parse_res {
            Ok(()) => {
                if show_help {
                    let mut launcher_opts = launcher::LauncherOptions::default();
                    let launcher_ap = launcher::arg_parser(&mut launcher_opts);
                    launcher_ap.print_help(&arg0, &mut stdout()).unwrap();
                    exit(0);
                }
                if show_version {
                    println!("{}", env!("VAGGA_VERSION"));
                    exit(0);
                }
            },
            _ => {},
        }
    }

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

#[cfg(feature="docker_runner")]
fn main() {
    init_logging();
    exit(launcher::run(env::args().collect()));
}
