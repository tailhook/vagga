use std::io::{stdout, stderr};
use std::default::Default;
use std::path::Path;
use std::fs::remove_file;
use rand;

use config::read_config;
use config::{Config, Container, Settings};
use argparse::{ArgumentParser, Store, StoreTrue};

use file_util::copy;
use self::context::{Context};
use self::commands::tarcmd::unpack_file;
use capsule::download::maybe_download_and_check_hashsum;
use container::util::clean_dir;
pub use self::guard::Guard;
pub use self::error::StepError;

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
}
mod packages;
mod timer;
mod distrib;
mod guard;
mod error;
mod dns;


pub fn run(input_args: Vec<String>) -> i32 {
    // Initialize thread random generator to avoid stack overflow (see #161)
    rand::thread_rng();

    let mut container_name: String = "".to_string();
    let mut settings: Settings = Default::default();
    let mut sources_only: bool = false;
    let mut ver: String = "".to_string();
    let mut no_image: bool = false;
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
        ap.refer(&mut no_image)
          .add_option(&["--no-image-download"], StoreTrue,
                "Do not download container image");
        match ap.parse(input_args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return 0,
            Err(_) => return 122,
        }
    }

    // TODO(tailhook) read also config from /work/.vagga/vagga.yaml
    let config = read_config(&Path::new("/work/vagga.yaml")).ok()
        .expect("Error parsing configuration file");  // TODO
    let container = config.containers.get(&container_name)
        .expect("Container not found");  // TODO

    if !no_image {
        if let Some(ref image_cache_url_tmpl) = container.image_cache_url {
            let short_hash = match ver.rsplitn(2, ".").next() {
                Some(v) => v,
                None => {
                    error!("Incorrect container version");
                    return 122;
                }
            };
            let image_cache_url = image_cache_url_tmpl
                .replace("${container_name}", &container_name)
                .replace("${short_hash}", &short_hash);
            let res = _build_from_image(&container_name, &container,
                                        &config, &settings, &image_cache_url);
            // just ignore errors if we cannot build from image
            match res {
                Ok(()) => return 0,
                Err(e) => {
                    error!("Error when unpacking image: {}. \
                            Will clean and build it locally...", e);
                }
            }
            match clean_dir("/vagga/root", false) {
                Ok(()) => {}
                Err(e) => {
                    error!("Can't clean invalid cache image: {}", e);
                    return 1;
                }
            }
        }
    }

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

fn _build_from_image(container_name: &String, container: &Container,
    config: &Config, settings: &Settings, image_cache_url: &String)
    -> Result<(), String>
{
    // TODO(tailhook) read also config from /work/.vagga/vagga.yaml
    let settings = settings.clone();
    let mut ctx = Context::new(config, container_name.clone(),
                               container, settings);

    let (filename, downloaded) = maybe_download_and_check_hashsum(
        &mut ctx.capsule, image_cache_url, None)?;
    match unpack_file(&mut ctx, &filename, &Path::new("/vagga/root"), &[], &[], true) {
        Ok(_) => {
            info!("Succesfully unpack image {}", image_cache_url);
            // If container is okay, we need to store uid_map used for
            // unpacking
            copy("/proc/self/uid_map", "/vagga/container/uid_map")
                .map_err(|e| format!("Error copying uid_map: {}", e))?;
            copy("/proc/self/gid_map", "/vagga/container/gid_map")
                .map_err(|e| format!("Error copying gid_map: {}", e))?;
            // Remove image from local cache after unpacking
            if downloaded {
                remove_file(&filename)
                    .map_err(|e| error!(
                        "Error unlinking cache file: {}", e)).ok();

            }
        },
        Err(e) => {
            return Err(format!("Error unpacking image {}: {}",
                image_cache_url, e));
        },
    }

    Ok(())
}

fn _build(container_name: &String, container: &Container,
          config: &Config, settings: &Settings)
          -> Result<(), String> {

    Guard::build(Context::new(config, container_name.clone(), container, settings.clone()))
    .map_err(|e| e.to_string())
}

fn _fetch_sources(_container: &Container, _settings: &Settings)
    -> Result<(), String>
{
    unimplemented!();
}
