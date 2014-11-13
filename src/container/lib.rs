#![crate_name="container"]
#![crate_type="lib"]
#![feature(if_let, phase, macro_rules)]

extern crate libc;
extern crate time;
#[phase(plugin, link)] extern crate log;

extern crate config;

pub mod util;
pub mod macros;

pub mod container;
pub mod monitor;
pub mod signal;
pub mod pipe;
pub mod uidmap;
pub mod mount;
pub mod root;
