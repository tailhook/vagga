#![feature(phase, if_let)]

extern crate quire;
extern crate argparse;
extern crate serialize;
extern crate regex;
#[phase(plugin)] extern crate regex_macros;

extern crate config;
#[phase(plugin, link)] extern crate container;

use std::os::{set_exit_status};

use argparse::{ArgumentParser, Store, List};


pub fn run() -> int {
    let mut container: String = "".to_string();
    let mut args: Vec<String> = Vec::new();
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
    println!("Container {}", container);
    return 2;
}

fn main() {
    let val = run();
    set_exit_status(val);
}
