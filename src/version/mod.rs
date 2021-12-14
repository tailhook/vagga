use std::default::Default;
use std::fs::File;
use std::path::Path;
use std::io::{stdout, stderr, Write};
use std::os::unix::io::FromRawFd;

use argparse::{ArgumentParser, Store, StoreTrue};

use crate::config::{find_config_or_exit, Settings};

mod version;
mod error;

#[cfg(feature="containers")]
pub use self::version::short_version;
pub use self::error::Error;


#[cfg(feature="containers")]
pub fn run(input_args: Vec<String>) -> i32 {
    let mut container: String = "".to_string();
    let mut settings: Settings = Default::default();
    let mut debug_info = false;
    let mut dump = false;
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
        ap.refer(&mut debug_info)
            .add_option(&["--debug-versioning"], StoreTrue, "
                Print debugging info for versioning to stdout
                (WARNING: may contain binary data)");
        ap.refer(&mut dump)
            .add_option(&["--dump-version-data"], StoreTrue, "
                Dump the data used for versioning directly to stdout
                (WARNING: contains binary data!)");
        match ap.parse(input_args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return 0,
            Err(_) => return 122,
        }
    }

    let (cfg, _) = find_config_or_exit(&Path::new("/work"), false);
    let cont = cfg.containers.get(&container)
        .expect(&format!("Container {:?} not found", &container));

    let hash = match version::long_version(&cont, &cfg, debug_info, dump) {
        Ok(hash) => hash,
        Err((_, Error::New))  => {
            return 29;
        }
        Err((cmd, e)) => {
            error!("Error versioning command {}: {}", cmd, e);
            return 1;
        }
    };

    debug!("Got hash {:?}", hash);
    match unsafe { File::from_raw_fd(3) }.write_all(hash.as_bytes()) {
        Ok(()) => {}
        Err(e) => {
            error!("Error writing hash: {}", e);
            return 1;
        }
    }
    return 0;
}
