extern crate serialize;
extern crate collections;
extern crate debug;

extern crate quire;
extern crate argparse;

use std::os::getcwd;
use std::os::set_exit_status;
use std::io::stdio::stderr;

use self::config::find_config;


mod config;

fn _main() -> int {
    let mut err = stderr();
    let workdir = getcwd();
    let (config, path) = match find_config(workdir) {
        Ok(tup) => tup,
        Err(e) => {
            err.write_line(e.as_slice()).ok();
            return 2;
        }
    };
    err.write_line("Available commands:").ok();
    for (k, v) in config.commands.iter() {
        err.write_str("    ").ok();
        err.write_line(k.as_slice()).ok();
    }
    return 1;
}

fn main() {
    set_exit_status(_main());
}
