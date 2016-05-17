use shaman::sha2::Sha256;
use shaman::digest::Digest as ShamanDigest;

use config::{Config, Container};
use super::error::Error;
use build_step::{BuildStep, Digest};


fn all(container: &Container, cfg: &Config)
    -> Result<Sha256, (String, Error)>
{
    debug!("Versioning items: {}", container.setup.len());

    let mut hash = Digest::new();

    hash.item("uids");
    for i in &container.uids {
        hash.item(&format!("{}-{}", i.start(), i.end()));
    }
    hash.item("gids");
    for i in &container.gids {
        hash.item(&format!("{}-{}", i.start(), i.end()));
    }

    for b in container.setup.iter() {
        debug!("Versioning setup: {:?}", b);
        try!(b.hash(&cfg, &mut hash).map_err(|e| (format!("{:?}", b), e)));
    }

    Ok(hash.unwrap())
}

pub fn short_version(container: &Container, cfg: &Config)
    -> Result<String, (String, Error)>
{
    let mut hash = try!(all(container, cfg));
    Ok(hash.result_str()[..8].to_string())
}

pub fn long_version(container: &Container, cfg: &Config)
    -> Result<String, (String, Error)>
{
    let mut hash = try!(all(&container, cfg));
    Ok(hash.result_str())
}
