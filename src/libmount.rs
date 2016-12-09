//! Dummy module used for mocking `libmount` crate on osx
#![allow(unused)]

use std::path::Path;
use std::fmt;

type mode_t = u32;
type uid_t = u32;
type gid_t = u32;

pub struct BindMount;
pub struct Tmpfs;
pub struct Overlay;
pub struct Remount;
pub struct Error;
pub struct OSError;

impl BindMount {
    pub fn new<A: AsRef<Path>, B: AsRef<Path>>(source: A, target: B)
        -> BindMount
    {
        unimplemented!();
    }
    pub fn recursive(mut self, flag: bool) -> BindMount { unimplemented!(); }
    pub fn readonly(mut self, flag: bool) -> BindMount { unimplemented!(); }
    pub fn bare_mount(self) -> Result<(), OSError> { unimplemented!(); }
    pub fn mount(self) -> Result<(), Error> { unimplemented!(); }
}

impl Tmpfs {
    pub fn new<P: AsRef<Path>>(path: P) -> Tmpfs { unreachable!(); }
    pub fn size_bytes(mut self, size: usize) -> Tmpfs { unreachable!(); }
    pub fn size_blocks(mut self, size: usize) -> Tmpfs { unreachable!(); }
    pub fn nr_inodes(mut self, num: usize) -> Tmpfs { unreachable!(); }
    pub fn mode(mut self, mode: mode_t) -> Tmpfs { unreachable!(); }
    pub fn uid(mut self, uid: uid_t) -> Tmpfs { unreachable!(); }
    pub fn gid(mut self, gid: gid_t) -> Tmpfs { unreachable!(); }
    pub fn bare_mount(self) -> Result<(), OSError> { unreachable!(); }
    pub fn mount(self) -> Result<(), Error> { unreachable!(); }
}

impl Overlay {
    pub fn readonly<'x, I, T>(dirs: I, target: T) -> Overlay
        where I: Iterator<Item=&'x Path>, T: AsRef<Path>
    {
        unimplemented!();
    }
    pub fn writable<'x, I, B, C, D>(lowerdirs: I, upperdir: B,
                                workdir: C, target: D)
        -> Overlay
        where I: Iterator<Item=&'x Path>, B: AsRef<Path>,
              C: AsRef<Path>, D: AsRef<Path>,
    {
        unimplemented!();
    }
    pub fn bare_mount(self) -> Result<(), OSError> { unimplemented!(); }
    pub fn mount(self) -> Result<(), Error> { unimplemented!(); }
}

impl Remount {
    pub fn new<P: AsRef<Path>>(path: P) -> Remount { unimplemented!(); }
    pub fn bind(mut self, flag: bool) -> Remount { unimplemented!(); }
    pub fn readonly(mut self, flag: bool) -> Remount { unimplemented!(); }
    pub fn bare_remount(self) -> Result<(), OSError> { unimplemented!(); }
    pub fn remount(self) -> Result<(), Error> { unimplemented!(); }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result { unreachable!(); }
}

impl fmt::Debug for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result { unreachable!(); }
}

impl OSError {
    pub fn explain(self) -> Error { unimplemented!(); }
}

impl fmt::Display for OSError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result { unimplemented!(); }
}
