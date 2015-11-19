use std::io;
use std::io::{BufReader, BufRead, Read};
use std::fs::File;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use shaman::sha2::Sha256;
use shaman::digest::Digest;
use rustc_serialize::json;

use config::{Config, Container};
use config::read_config;
use config::builders::{Builder};
use config::builders::Builder as B;
use config::builders::Source as S;
use self::Error::{New, ContainerNotFound};
use path_util::PathExt;


quick_error! {
    /// Versioning error
    #[derive(Debug)]
    pub enum Error {
        /// Hash sum can't be calculated because some files need to be
        /// generated during build
        New {
            description("dependencies are not ready")
        }
        /// Some error occured. Unfortunately all legacy errors are strings
        String(s: String) {
            from()
            description("error versioning dependencies")
            display("version error: {}", s)
        }
        /// I/O error
        Io(err: io::Error, path: PathBuf) {
            cause(err)
            description("io error")
            display("Error reading {:?}: {}", path, err)
        }
        /// Container needed for build is not found
        ContainerNotFound(name: String) {
            description("container not found")
            display("container {:?} not found", name)
        }
        /// Some step of subcontainer failed
        SubStepError(step: String, err: Box<Error>) {
            from(tuple: (String, Error)) -> (tuple.0, Box::new(tuple.1))
        }
    }
}

pub trait VersionHash {
    fn hash(&self, cfg: &Config, hash: &mut Digest) -> Result<(), Error>;
}


impl VersionHash for Builder {
    fn hash(&self, cfg: &Config, hash: &mut Digest) -> Result<(), Error> {
        match self {
            &B::Py2Requirements(ref fname) | &B::Py3Requirements(ref fname)
            => {
                let path = Path::new("/work").join(fname);
                let err = |e| Error::Io(e, path.clone());
                File::open(&path)
                .and_then(|f| {
                        let f = BufReader::new(f);
                        for line in f.lines() {
                            let line = try!(line);
                            let chunk = line[..].trim();
                            // Ignore empty lines and comments
                            if chunk.len() == 0 || chunk.starts_with("#") {
                                continue;
                            }
                            // Should we also ignore the order?
                            hash.input(chunk.as_bytes());
                        }
                        Ok(())
                }).map_err(err)
            }
            &B::PyFreeze(_) => unimplemented!(),
            &B::Depends(ref filename) => {
                let path = Path::new("/work").join(filename);
                let err = |e| Error::Io(e, path.clone());
                File::open(&path)
                .and_then(|mut f| {
                    loop {
                        let mut chunk = [0u8; 8*1024];
                        let bytes = match f.read(&mut chunk[..]) {
                            Ok(0) => break,
                            Ok(bytes) => bytes,
                            Err(e) => return Err(e),
                        };
                        hash.input(&chunk[..bytes]);
                    }
                    Ok(())
                }).map_err(err)
            }
            &B::Container(ref name) => {
                let cont = try!(cfg.containers.get(name)
                    .ok_or(ContainerNotFound(name.to_string())));
                for b in cont.setup.iter() {
                    debug!("Versioning setup: {:?}", b);
                    try!(b.hash(cfg, hash));
                }
                Ok(())
            }
            &B::SubConfig(ref sconfig) => {
                let path = match sconfig.source {
                    S::Container(ref container) => {
                        let cinfo = try!(cfg.containers.get(container)
                            .ok_or(ContainerNotFound(container.clone())));
                        let version = try!(short_version(&cinfo, cfg));
                        Path::new("/vagga/base/.roots")
                            .join(format!("{}.{}", container, version))
                            .join("root").join(&sconfig.path)
                    }
                    S::Git(ref _git) => {
                        unimplemented!();
                    }
                    S::Directory => {
                        Path::new("/work").join(&sconfig.path)
                    }
                };
                if !path.exists() {
                    return Err(New);
                }
                let subcfg = try!(read_config(&path));
                let cont = try!(subcfg.containers.get(&sconfig.container)
                    .ok_or(ContainerNotFound(sconfig.container.to_string())));
                for b in cont.setup.iter() {
                    debug!("Versioning setup: {:?}", b);
                    try!(b.hash(cfg, hash));
                }
                Ok(())
            }
            &B::CacheDirs(ref map) => {
                for (k, v) in map.iter() {
                    hash.input(k.as_os_str().as_bytes());
                    hash.input(b"\0");
                    hash.input(v.as_bytes());
                    hash.input(b"\0");
                }
                Ok(())
            }
            &B::Text(ref map) => {
                for (k, v) in map.iter() {
                    hash.input(k.as_os_str().as_bytes());
                    hash.input(b"\0");
                    hash.input(v.as_bytes());
                    hash.input(b"\0");
                }
                Ok(())
            }
            _ => {
                hash.input(json::encode(self).unwrap().as_bytes());
                Ok(())
            }
        }
    }
}

fn all(setup: &[Builder], cfg: &Config)
    -> Result<Sha256, (String, Error)>
{
    debug!("Versioning items: {}", setup.len());

    let mut hash = Sha256::new();

    let mut buf = Vec::with_capacity(1000);
    File::open(&Path::new("/proc/self/uid_map"))
               .and_then(|mut f| f.read_to_end(&mut buf))
               .ok().expect("Can't read uid_map");
    hash.input(&buf);

    let mut buf = Vec::with_capacity(1000);
    File::open(&Path::new("/proc/self/gid_map"))
               .and_then(|mut f| f.read_to_end(&mut buf))
               .ok().expect("Can't read gid_map");
    hash.input(&buf);

    for b in setup.iter() {
        debug!("Versioning setup: {:?}", b);
        try!(b.hash(&cfg, &mut hash).map_err(|e| (format!("{:?}", b), e)));
    }

    Ok(hash)
}

pub fn short_version(container: &Container, cfg: &Config)
    -> Result<String, (String, Error)>
{
    let mut hash = try!(all(&container.setup, cfg));
    Ok(hash.result_str()[..8].to_string())
}

pub fn long_version(container: &Container, cfg: &Config)
    -> Result<String, (String, Error)>
{
    let mut hash = try!(all(&container.setup, cfg));
    Ok(hash.result_str())
}
