use std::path::Path;


pub mod v1 {
    use std::io;
    use super::ScannerConfig;

    pub fn scan<F>(config: &ScannerConfig, out: &mut F) -> Result<(), String>
        where F: io::Write
    {
        unimplemented!();
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

