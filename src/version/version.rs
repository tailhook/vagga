use std::io::EndOfFile;
use std::io::BufferedReader;
use std::io::fs::File;

use serialize::json;

use config::builders::{Builder};
use config::builders::Builder as B;
use container::sha256::Digest;
use self::HashResult::*;


pub enum HashResult {
    Hashed,
    New,
    Error(String)
}


pub trait VersionHash {
    fn hash(&self, hash: &mut Digest) -> HashResult;
}


impl VersionHash for Builder {
    fn hash(&self, hash: &mut Digest) -> HashResult {
        match self {
            &B::Py2Requirements(ref fname) | &B::Py3Requirements(ref fname)
            => {
                match
                    File::open(&Path::new("/work").join(fname))
                    .and_then(|f| {
                        let mut f = BufferedReader::new(f);
                        loop {
                            let line = match f.read_line() {
                                Ok(line) => line,
                                Err(ref e) if e.kind == EndOfFile => {
                                    break;
                                }
                                Err(e) => {
                                    return Err(e);
                                }
                            };
                            let chunk = line.as_slice().trim();
                            // Ignore empty lines and comments
                            if chunk.len() == 0 || chunk.starts_with("#") {
                                continue;
                            }
                            // Should we also ignore the order?
                            hash.input(chunk.as_bytes());
                        }
                        Ok(())
                    })
                {
                    Err(e) => return Error(format!("Can't read file: {}", e)),
                    Ok(()) => return Hashed,
                }
            }
            &B::Depends(ref filename) => {
                match
                    File::open(&Path::new("/work").join(filename))
                    .and_then(|mut f| {
                        loop {
                            let mut chunk = [0u8; 128*1024];
                            let bytes = match f.read(chunk.as_mut_slice()) {
                                Ok(bytes) => bytes,
                                Err(ref e) if e.kind == EndOfFile => break,
                                Err(e) => return Err(e),
                            };
                            hash.input(chunk[..bytes].as_slice());
                        }
                        Ok(())
                    })
                {
                    Err(e) => return Error(format!("Can't read file: {}", e)),
                    Ok(()) => return Hashed,
                }
            }
            _ => {
                hash.input(json::encode(self).as_bytes());
                Hashed
            }
        }
    }
}
