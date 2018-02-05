use std::fs::File;
use std::io::Read;
use std::path::Path;

use failure::{Error, ResultExt};
use resolv_conf::{self, ScopedIp};


pub fn get_nameservers() -> Result<Vec<ScopedIp>, Error> {
    let mut buf = Vec::with_capacity(1024);
    File::open(&Path::new("/etc/resolv.conf"))
        .and_then(|mut f| f.read_to_end(&mut buf))
        .context("error reading resolv.conf")?;
    let config = resolv_conf::Config::parse(&buf)
        .context("error reading resolv.conf")?;
    return Ok(config.nameservers);
}

pub fn detect_local_dns() -> Result<Option<String>, Error> {
    let nameservers = get_nameservers()?;
    info!("Detected nameservers: {:?}", nameservers);

    let local_dns = match nameservers.first() {
        Some(&ScopedIp::V4(ip))
        if nameservers.len() == 1 && ip.octets()[..3] == [127, 0, 0]
        => Some(ip.to_string()),
        _ => None,
    };
    Ok(local_dns)
}
