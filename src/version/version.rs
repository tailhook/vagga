use std::io::Read;
use std::fs::File;
use std::path::Path;

use shaman::sha2::Sha256;
use shaman::digest::Digest as ShamanDigest;

use config::{Config, Container};
use super::error::Error;
use build_step::{Step, BuildStep, Digest};


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
