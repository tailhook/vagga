use std::fmt;
use std::io::{self, Read, Seek};
use std::path::Path;


pub mod v1 {
    use std::fmt;
    use std::io;
    use super::ScannerConfig;

    pub struct Entry;
    pub struct EntryKind;
    #[derive(PartialEq, Eq, Hash)]
    pub struct Hashes;
    pub struct Parser;
    #[derive(Debug)]
    pub struct ParseError;

    impl fmt::Display for ParseError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            unimplemented!()
        }
    }

    pub fn scan<F>(config: &ScannerConfig, out: &mut F) -> Result<(), String>
        where F: io::Write
    {
        unimplemented!();
    }

    pub mod merge {
        pub struct FileMergeBuilder;
        #[derive(Debug)]
        pub struct MergeError;
    }
}

#[derive(Debug)]
pub struct Error;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
}

pub enum HashType {
    Blake2b_256,
}

pub struct ScannerConfig {
}

impl ScannerConfig {
    pub fn new() -> ScannerConfig {
        unimplemented!();
    }
    pub fn hash(&mut self, _:HashType) -> &mut Self {
        unimplemented!();
    }
    pub fn add_dir<P, R>(&mut self, path: P, prefix: R) -> &mut Self
        where P: AsRef<Path>, R: AsRef<Path>
    {
        unimplemented!();
    }
}

pub fn get_hash<F: Read + Seek>(_: &mut F) -> Result<Vec<u8>, io::Error> {
    unimplemented!();
}
