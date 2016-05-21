use std::io::{stdout, stderr};

use argparse::{ArgumentParser, Store, StoreTrue};


pub struct Options {
    pub container: String,
    pub short: bool,
    pub fd3: bool,
}

impl Options {
    pub fn parse(args: &Vec<String>, internal: bool)
        -> Result<Options, i32>
    {
        let mut opt = Options {
            container: String::from(""),
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
