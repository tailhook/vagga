use std::io::{stdout, stderr};
use std::default::Default;
use std::path::Path;

use argparse::{ArgumentParser, Store, StoreTrue};
use rand;

use crate::config::{Config, Container, find_config_or_exit, Settings};


#[cfg(feature="containers")]
pub use self::{
    guard::Guard,
    error::StepError,
};

#[cfg(feature="containers")]
pub mod context;
pub mod commands {
    pub mod ubuntu;
    pub mod generic;
    pub mod alpine;
    pub mod pip;
    pub mod gem;
    pub mod npm;
    pub mod composer;
    pub mod vcs;
    pub mod download;
    pub mod subcontainer;
    pub mod copy;
    pub mod text;
    pub mod dirs;
    pub mod packaging;
    pub mod tarcmd;
    pub mod unzip;
    pub mod docker;
}
#[cfg(feature="containers")] pub mod guard;
#[cfg(feature="containers")] mod packages;
#[cfg(feature="containers")] mod timer;
#[cfg(feature="containers")] mod distrib;
#[cfg(feature="containers")] mod error;
#[cfg(feature="containers")] mod dns;


// TODO(tailhook) remove this when we can get rid of unneeded methods in
//                BuildStep
#[cfg(not(feature="containers"))]
pub struct Guard {
    _private: ()
}

// TODO(tailhook) remove this when we can get rid of unneeded methods in
//                BuildStep
#[cfg(not(feature="containers"))]
pub struct StepError {
    _private: ()
}

#[cfg(feature="containers")]
pub fn run(input_args: Vec<String>) -> i32 {
    // Initialize thread random generator to avoid stack overflow (see #161)
    rand::thread_rng();

    let mut container_name: String = "".to_string();
    let mut settings: Settings = Default::default();
    let mut sources_only: bool = false;
    let mut ver: String = "".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            A tool which builds containers
            ");
        ap.refer(&mut container_name)
          .add_argument("container", Store,
                "A container to version")
          .required();
        ap.refer(&mut sources_only)
          .add_option(&["--sources-only"], StoreTrue,
                "Only fetch sources, do not build container");
        ap.refer(&mut settings)
          .add_option(&["--settings"], Store,
                "User settings for the container build");
        ap.refer(&mut ver)
          .add_option(&["--container-version"], Store,
                "Version for the container build");
        match ap.parse(input_args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return 0,
            Err(_) => return 122,
        }
    }

    let (config, _) = find_config_or_exit(&Path::new("/work"), false);
    let container = config.containers.get(&container_name)
        .expect(&format!("Container {:?} not found", &container_name));

    if sources_only {
        _fetch_sources(&container, &settings)
            .map(|()| 0)
            .map_err(|e| error!("Error fetching sources {:?}: {}",
                                container_name, e))
            .unwrap_or(1)
    } else {
        _build(&container_name, &container, &config, &settings)
            .map(|()| 0)
            .map_err(|e| error!("Error building container {:?}: {}",
                                container_name, e))
            .unwrap_or(1)
    }
}

#[cfg(feature="containers")]
fn _build(container_name: &String, container: &Container,
          config: &Config, settings: &Settings)
          -> Result<(), String> {

    Guard::build(context::Context::new(config,
        container_name.clone(), container, settings.clone()))
    .map_err(|e| e.to_string())
}

#[cfg(feature="containers")]
fn _fetch_sources(_container: &Container, _settings: &Settings)
    -> Result<(), String>
{
    unimplemented!();
}
