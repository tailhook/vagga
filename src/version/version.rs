use std::env;

use config::{Config, Container};
use super::error::Error;
use build_step::{BuildStep, Digest};


fn all(container: &Container, cfg: &Config)
    -> Result<String, (String, Error)>
{
    debug!("Versioning items: {}", container.setup.len());

    let mut hash = Digest::new(
        env::var("VAGGA_DEBUG_HASH").map(|x| x.len() > 0).unwrap_or(false),
        env::var("VAGGA_DUMP_HASH").map(|x| x.len() > 0).unwrap_or(false));

    hash.field("uids", &container.uids);
    hash.field("gids", &container.gids);

    for b in container.setup.iter() {
        debug!("Versioning setup: {:?}", b);
        hash.command(b.name());
        b.hash(&cfg, &mut hash).map_err(|e| (format!("{:?}", b), e))?;
    }

    if !container.data_dirs.is_empty() {
        hash.field("data_dirs", &container.data_dirs);
    }

    Ok(format!("{:x}", hash))
}

pub fn short_version(container: &Container, cfg: &Config)
    -> Result<String, (String, Error)>
{
    let hash_str = all(container, cfg)?;
    Ok(hash_str[..8].to_string())
}

pub fn long_version(container: &Container, cfg: &Config)
    -> Result<String, (String, Error)>
{
    let hash_str = all(&container, cfg)?;
    Ok(hash_str)
}
