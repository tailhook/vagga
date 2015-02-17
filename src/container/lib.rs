#![crate_name="container"]
#![crate_type="lib"]
#![feature(slicing_syntax)]

extern crate libc;
extern crate serialize;
extern crate collections;
#[macro_use] extern crate log;

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
pub mod async;
pub mod sha256;
pub mod nsutil;
pub mod vagga;
