use std::old_io::EndOfFile;
use std::old_io::BufferedReader;
use std::old_io::fs::File;
use std::old_path::BytesContainer;

use serialize::json;

use config::Config;
use config::read_config;
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
    fn hash(&self, cfg: &Config, hash: &mut Digest) -> HashResult;
}


impl VersionHash for Builder {
    fn hash(&self, cfg: &Config, hash: &mut Digest) -> HashResult {
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
            &B::Container(ref name) => {
                let cont = match cfg.containers.get(name) {
                    Some(cont) => cont,
                    None => {
                        return Error(format!("Container {:?} not found",
                                             name));
                    }
                };
                for b in cont.setup.iter() {
                    debug!("Versioning setup: {:?}", b);
                    match b.hash(cfg, hash) {
                        Hashed => continue,
                        New => return New,  // Always rebuild
                        Error(e) => {
                            return Error(format!("{:?}: {}", name, e));
                        }
                    }
                }
                Hashed
            }
            &B::SubConfig(ref sconfig) => {
                assert!(sconfig.generator.is_none());
                let subcfg = match read_config(
                    &Path::new("/work").join(&sconfig.path))
                {
                    Ok(cfg) => cfg,
                    Err(e) => return Error(e),
                };
                let cont = match subcfg.containers.get(&sconfig.container) {
                    Some(cont) => cont,
                    None => {
                        return Error(format!(
                            "Container {:?} not found in {:?}",
                            sconfig.container, sconfig.path));
                    }
                };
                for b in cont.setup.iter() {
                    debug!("Versioning setup: {:?}", b);
                    match b.hash(cfg, hash) {
                        Hashed => continue,
                        New => return New,  // Always rebuild
                        Error(e) => {
                            return Error(format!("{:?}: {}",
                                sconfig.container, e));
                        }
                    }
                }
                Hashed
            }
            &B::CacheDirs(ref map) => {
                for (k, v) in map.iter() {
                    hash.input(k.container_as_bytes());
                    hash.input(b"\0");
                    hash.input(v.as_bytes());
                    hash.input(b"\0");
                }
                Hashed
            }
            &B::Text(ref map) => {
                for (k, v) in map.iter() {
                    hash.input(k.container_as_bytes());
                    hash.input(b"\0");
                    hash.input(v.as_bytes());
                    hash.input(b"\0");
                }
                Hashed
            }
            _ => {
                hash.input(json::encode(self).unwrap().as_bytes());
                Hashed
            }
        }
    }
}
