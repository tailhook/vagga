use std::default::Default;
use std::path::Path;
use std::process::exit;

use config::read_config;
use config::Settings;
use config::builders::Builder as B;
use config::builders::Source as S;
use argparse::{ArgumentParser, Store, StoreTrue};
use self::context::{BuildContext};
use self::bld::{BuildCommand};

mod context;
mod bld;
mod download;
mod tarcmd;
mod commands {
    pub mod debian;
    pub mod generic;
    pub mod alpine;
    pub mod pip;
    pub mod npm;
    pub mod vcs;
}
mod capsule;
mod packages;
mod timer;


pub fn run() -> i32 {
    let mut container: String = "".to_string();
    let mut settings: Settings = Default::default();
    let mut sources_only: bool = false;
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            A tool which builds containers
            ");
        ap.refer(&mut container)
          .add_argument("container", Store,
                "A container to version")
          .required();
        ap.refer(&mut sources_only)
          .add_option(&["--sources-only"], StoreTrue,
                "Only fetch sources, do not build container");
        ap.refer(&mut settings)
          .add_option(&["--settings"], Store,
                "User settings for the container build");
        match ap.parse_args() {
            Ok(()) => {}
            Err(0) => return 0,
            Err(_) => return 122,
        }
    }

    if sources_only {
        _fetch_sources(&container, settings)
            .map(|()| 0)
            .map_err(|e| error!("Error fetching sources {:?}: {}",
                                container, e))
            .unwrap_or(1)
    } else {
        _build(&container, settings)
            .map(|()| 0)
            .map_err(|e| error!("Error building container {:?}: {}",
                                container, e))
            .unwrap_or(1)
    }
}

fn _build(container: &String, settings: Settings) -> Result<(), String> {
    // TODO(tailhook) read also config from /work/.vagga/vagga.yaml
    let cfg = read_config(&Path::new("/work/vagga.yaml")).ok()
        .expect("Error parsing configuration file");  // TODO
    let cont = cfg.containers.get(container)
        .expect("Container not found");  // TODO

    let mut build_context = BuildContext::new(
        &cfg, container.clone(), cont, settings);
    try!(build_context.start());

    for b in cont.setup.iter() {
        debug!("Building step: {:?}", b);
        try!(b.build(&mut build_context, true));
    }

    try!(build_context.finish());
    Ok(())
}

fn _fetch_sources(container: &String, settings: Settings)
    -> Result<(), String>
{
    // TODO(tailhook) read also config from /work/.vagga/vagga.yaml
    let cfg = read_config(&Path::new("/work/vagga.yaml")).ok()
        .expect("Error parsing configuration file");  // TODO
    let cont = cfg.containers.get(container)
        .expect("Container not found");  // TODO
    let mut caps = Default::default();

    for b in cont.setup.iter() {
        match b {
            &B::SubConfig(ref cfg) => {
                if let S::Git(ref git) = cfg.source {
                    try!(commands::vcs::fetch_git_source(
                        &mut caps, &settings, git));
                }
            }
            _ => {}
        }
    }

    Ok(())
}

pub fn main() {
    let val = run();
    exit(val);
}
