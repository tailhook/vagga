use std::io::{stdout, stderr};

use argparse::{ArgumentParser, Store, StoreTrue};


pub struct Options {
    pub container: String,
    pub short: bool,
    pub debug_versioning: bool,
    pub dump_version_data: bool,
    pub fd3: bool,
}

impl Options {
    pub fn for_container(container: &str) -> Options {
        Options {
            container: container.to_string(),
            short: false,
            debug_versioning: false,
            dump_version_data: false,
            fd3: false,
        }
    }
    pub fn parse(args: &Vec<String>, internal: bool)
        -> Result<Options, i32>
    {
        let mut opt = Options {
            container: String::from(""),
            debug_versioning: false,
            dump_version_data: false,
            short: false,
            fd3: false,
        };
        {
            let mut ap = ArgumentParser::new();
            ap.set_description("
                Prints version hash of the container without building it. If
                this command exits with code 29, then container can't be
                versioned before the build.
                ");
            ap.refer(&mut opt.container)
                .add_argument("container_name", Store,
                    "Container name to build")
                .required();
            ap.refer(&mut opt.short)
                .add_option(&["-s", "--short"], StoreTrue, "
                    Print short container version, like used in directory
                    names (8 chars)");
            ap.refer(&mut opt.debug_versioning)
                .add_option(&["--debug-versioning"], StoreTrue, "
                    Print debugging info for versioning to stdout
                    (WARNING: may contain binary data)");
            ap.refer(&mut opt.dump_version_data)
                .add_option(&["--dump-version-data"], StoreTrue, "
                    Dump the data used for versioning directly to stdout
                    (WARNING: contains binary data!)");
            if internal {
                ap.refer(&mut opt.fd3)
                    .add_option(&["--fd3"], StoreTrue,
                        "Print into file descriptor #3 instead of stdout");
            }
            match ap.parse(args.clone(), &mut stdout(), &mut stderr()) {
                Ok(()) => {}
                Err(0) => return Err(0),
                Err(_) => return Err(122),
            }
        }
        return Ok(opt);
    }
}
