use std::default::Default;
use std::path::{Path, PathBuf};
use std::process::exit;

use argparse::{ArgumentParser, ParseOption, Store};
use unshare::{Command, Stdio};

use builder::capsule;
use builder::context::Context;
use config::read_config;
use config::Settings;
use options::compression_type::{compression_type, CompressionType};
use process_util::convert_status;


pub fn run() -> i32 {
    let mut container: String = "".to_string();
    let mut file_path: Option<PathBuf> = None;
    let mut compression: Option<CompressionType> = None;
    let mut settings: Settings = Default::default();
    let mut ver: String = "".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            A tool which push images into image registry
            ");
        ap.refer(&mut container)
          .add_argument("container", Store,
                "A container to pack")
          .required();
        ap.refer(&mut file_path)
            .add_option(&["-f", "--file"], ParseOption,
                "File to store tar archive at");
        compression_type(&mut ap, &mut compression);
        ap.refer(&mut settings)
          .add_option(&["--settings"], Store,
                "User settings for the container");
        ap.refer(&mut ver)
          .add_option(&["--container-version"], Store,
                "Version of the container to push");
        match ap.parse_args() {
            Ok(()) => {}
            Err(0) => return 0,
            Err(_) => return 122,
        }
    }

    pack_image(&file_path, &compression, &container, &ver, &settings)
        .map(|()| 0)
        .map_err(|e| error!("{}", e))
        .unwrap_or(1)
}

fn pack_image(image_path: &Option<PathBuf>,
    compression_type: &Option<CompressionType>, container: &String,
    version: &String, settings: &Settings)
    -> Result<(), String>
{
    let cfg = read_config(&Path::new("/work/vagga.yaml")).ok()
        .expect("Error parsing configuration file");
    let cont = cfg.containers.get(container)
        .expect("Container not found");

    let mut capsule_features = vec!();
    let root = Path::new("/vagga/base/.roots")
        .join(version)
        .join("root");
    let mut tar_cmd = Command::new("/vagga/bin/busybox");
    tar_cmd.stdin(Stdio::null())
        .arg("tar")
        .arg("-c");
    if let Some(compression_type) = *compression_type {
        tar_cmd.arg(compression_type.get_short_option());
        capsule_features.push(compression_type.get_capsule_feature());
    }
    if let Some(ref image_path) = *image_path {
        tar_cmd.arg("-f").arg(image_path);
    }
    tar_cmd
        .arg("-C").arg(&root)
        .arg(".");

    if capsule_features.len() > 0 {
        let mut ctx = Context::new(&cfg, container.clone(), cont, settings.clone());
        try!(capsule::ensure_features(&mut ctx, &capsule_features, Some(2)));
    }

    info!("Running {:?}", tar_cmd);
    match tar_cmd.status() {
        Ok(st) if convert_status(st) > 0 => {
            return Err(format!("Error when archiving container {}: {:?}", container, tar_cmd));
        },
        Err(e) => {
            return Err(format!("Error when archiving container {}: {}", container, e));
        },
        _ => {},
    }

    Ok(())
}

pub fn main() {
    let val = run();
    exit(val);
}
