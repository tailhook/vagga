use std::default::Default;
use std::fs::File;
use std::path::Path;
use std::process::exit;
use std::io::{Read, Write};
use std::os::unix::io::FromRawFd;

use nix::unistd::dup2;
use argparse::{ArgumentParser, Store};
use shaman::sha2::Sha256;
use shaman::digest::Digest;

use config::read_config;
use config::Settings;
use self::version::{VersionHash};
use self::version::HashResult::{Hashed, New, Error};


mod version;


pub fn run() -> i32 {
    let mut container: String = "".to_string();
    let mut settings: Settings = Default::default();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            A tool which versions containers
            ");
        ap.refer(&mut container)
          .add_argument("container", Store,
                "A container to version")
          .required();
        ap.refer(&mut settings)
          .add_option(&["--settings"], Store,
                "User settings for the container build");
        match ap.parse_args() {
            Ok(()) => {}
            Err(0) => return 0,
            Err(_) => return 122,
        }
    }

    // TODO(tailhook) read also config from /work/.vagga/vagga.yaml
    let cfg = read_config(&Path::new("/work/vagga.yaml")).ok()
        .expect("Error parsing configuration file");  // TODO
    let cont = cfg.containers.get(&container)
        .expect("Container not found");  // TODO
    debug!("Versioning items: {}", cont.setup.len());

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

    for b in cont.setup.iter() {
        debug!("Versioning setup: {:?}", b);
        match b.hash(&cfg, &mut hash) {
            Hashed => continue,
            New => return 29,  // Always rebuild
            Error(e) => {
                error!("Error versioning command {:?}: {}", b, e);
                return 1;
            }
        }
    }
    debug!("Got hash {:?}", hash.result_str());
    match unsafe { File::from_raw_fd(3) }.write_all(hash.result_str().as_bytes()) {
        Ok(()) => {}
        Err(e) => {
            error!("Error writing hash: {}", e);
            return 1;
        }
    }
    return 0;
}

pub fn main() {
    // let's make stdout safer
    dup2(1, 3);
    dup2(2, 1);

    let val = run();
    exit(val);
}
