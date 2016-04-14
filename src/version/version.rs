use std::io::{BufReader, BufRead, Read};
use std::io::ErrorKind;
use std::fs::{File, symlink_metadata};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use rustc_serialize::json::{self, Json};
use regex::Regex;
use scan_dir::ScanDir;
use shaman::sha2::Sha256;
use shaman::digest::Digest as ShamanDigest;

use config::{Config, Container};
use config::read_config;
use path_util::ToRelative;
use super::error::Error::{self, New, ContainerNotFound};
use super::managers::{bundler};
use build_step::{Step, BuildStep, Digest};

/*
impl VersionHash for Builder {
    fn hash(&self, cfg: &Config, hash: &mut Digest) -> Result<(), Error> {
        match self {
            &B::GemBundle(ref info) => bundler::hash(info, hash),
            &B::ComposerDependencies(ref info) => composer::hash(info, hash),
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
            &B::Copy(ref cinfo) => {
                let ref src = cinfo.source;
                if src.starts_with("/work") {
                    match symlink_metadata(src) {
                        Ok(ref meta) if meta.file_type().is_dir() => {
                            let re = try!(Regex::new(&cinfo.ignore_regex)
                                .map_err(|e| Error::Regex(Box::new(e))));
                            try!(ScanDir::all().walk(src, |iter| {
                                let mut all_entries = iter.filter_map(|(e, _)|
                                {
                                    let fpath = e.path();
                                    let strpath = {
                                        // We know that directory is inside
                                        // the source
                                        let path = fpath.rel_to(src).unwrap();
                                        // We know that it's decodable
                                        let strpath = path.to_str().unwrap();
                                        if !re.is_match(strpath) {
                                            Some(strpath.to_string())
                                        } else {
                                            None
                                        }
                                    };
                                    strpath.map(|x| (fpath, x))
                                }).collect::<Vec<_>>();
                                all_entries.sort();
                                for (fpath, name) in all_entries {
                                    hash.input(b"\0");
                                    hash.input(name.as_bytes());
                                    hash.input(b"\0");
                                    try!(hash_file(&fpath, hash,
                                            cinfo.owner_uid, cinfo.owner_gid)
                                        .map_err(|e| Error::Io(e, fpath)));
                                }
                                Ok(())
                            }).map_err(Error::ScanDir).and_then(|x| x));
                        }
                        Ok(_) => {
                            try!(hash_file(src, hash,
                                    cinfo.owner_uid, cinfo.owner_gid)
                                .map_err(|e| Error::Io(e, src.into())));
                        }
                        Err(ref e) if e.kind() == ErrorKind::NotFound => {
                            return Err(Error::New);
                        }
                        Err(e) => {
                            return Err(Error::Io(e, src.into()));
                        }

                    }
                } else {
                    // We don't version the files outside of the /work because
                    // we believe they are result of the commands run above
                    //
                    // And we need already built container to version the files
                    // inside the container which is ugly
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
*/

fn all(setup: &[Step], cfg: &Config)
    -> Result<Sha256, (String, Error)>
{
    debug!("Versioning items: {}", setup.len());

    let mut hash = Digest::new();

    let mut buf = Vec::with_capacity(1000);
    File::open(&Path::new("/proc/self/uid_map"))
               .and_then(|mut f| f.read_to_end(&mut buf))
               .ok().expect("Can't read uid_map");
    hash.field("uid_map", &buf);

    let mut buf = Vec::with_capacity(1000);
    File::open(&Path::new("/proc/self/gid_map"))
               .and_then(|mut f| f.read_to_end(&mut buf))
               .ok().expect("Can't read gid_map");
    hash.field("gid_map", &buf);

    for b in setup.iter() {
        debug!("Versioning setup: {:?}", b);
        try!(b.hash(&cfg, &mut hash).map_err(|e| (format!("{:?}", b), e)));
    }

    Ok(hash.unwrap())
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
