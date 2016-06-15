use std::default::Default;
use std::path::{PathBuf, Path};

use std::collections::BTreeMap;
use rustc_serialize::{Decoder};

use quire::parse_config;
use quire::validate as V;

use super::containers;
use super::containers::Container;
use super::command::{MainCommand, command_validator};
use super::range::Range;
use super::validate::validate_config;
use super::version::MinimumVagga;

#[derive(RustcDecodable)]
pub struct Config {
    pub minimum_vagga: Option<String>,
    pub commands: BTreeMap<String, MainCommand>,
    pub containers: BTreeMap<String, Container>,
}

impl Config {
    pub fn get_container(&self, name: &str) -> Result<&Container, String> {
        self.containers.get(name)
        .ok_or_else(|| format!("Container {:?} not found", name))
    }
}

pub fn config_validator<'a>() -> V::Structure<'a> {
    V::Structure::new()
    .member("minimum_vagga", MinimumVagga::new()
        .optional()
        .current_version(env!("VAGGA_VERSION").to_string()))
    .member("containers", V::Mapping::new(
        V::Scalar::new(),
        containers::container_validator()))
    .member("commands", V::Mapping::new(
        V::Scalar::new(),
        command_validator()))
}

fn find_config_path(work_dir: &PathBuf) -> Option<(PathBuf, PathBuf)> {
    let mut dir = work_dir.clone();
    loop {
        let fname = dir.join(".vagga/vagga.yaml");
        if fname.exists() {
            return Some((dir, fname));
        }
        let fname = dir.join("vagga.yaml");
        if fname.exists() {
            return Some((dir, fname));
        }
        if !dir.pop() {
            return None;
        }
    }
}

pub fn find_config(work_dir: &PathBuf) -> Result<(Config, PathBuf), String> {
    let (cfg_dir, filename) = match find_config_path(work_dir) {
        Some(pair) => pair,
        None => return Err(format!(
            "Config not found in path {:?}", work_dir)),
    };
    assert!(cfg_dir.is_absolute());
    let cfg = try!(read_config(&filename));
    try!(validate_config(&cfg));
    return Ok((cfg, cfg_dir));
}

pub fn read_config(filename: &Path) -> Result<Config, String> {
    let mut config: Config = match parse_config(
        filename, &config_validator(), Default::default())
    {
        Ok(cfg) => cfg,
        Err(e) => {
            return Err(format!("Config {:?} cannot be read: {}",
                filename, e));
        }
    };
    for (_, ref mut container) in config.containers.iter_mut() {
        if container.uids.len() == 0 {
            container.uids.push(Range::new(0, 65535));
        }
        if container.gids.len() == 0 {
            container.gids.push(Range::new(0, 65535));
        }
    }
    return Ok(config);
}
