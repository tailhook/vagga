use std::os::unix::ffi::OsStrExt;

use config::{Config, Container};
use super::error::Error;
use build_step::{BuildStep, Digest};


fn all(container: &Container, cfg: &Config)
    -> Result<String, (String, Error)>
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

    if !container.data_dirs.is_empty() {
        let str_data_dirs = container.data_dirs.iter()
            .map(|p| p.as_os_str().as_bytes());
        hash.sequence("data_dirs", str_data_dirs);
    }

    Ok(hash.result_str())
}

pub fn short_version(container: &Container, cfg: &Config)
    -> Result<String, (String, Error)>
{
    let hash_str = try!(all(container, cfg));
    Ok(hash_str[..8].to_string())
}

pub fn long_version(container: &Container, cfg: &Config)
    -> Result<String, (String, Error)>
{
    let hash_str = try!(all(&container, cfg));
    Ok(hash_str)
}
