use std::default::Default;
use std::collections::BTreeMap;
use std::path::PathBuf;

use quire::validate as V;
use rustc_serialize::{Decoder};

use super::builders::{builder_validator};
use super::volumes::{Volume, volume_validator};
use super::Range;
use build_step::{Step};


#[derive(RustcDecodable, Clone)]
pub struct Container {
    pub setup: Vec<Step>,
    pub image_cache_url: Option<String>,
    pub auto_clean: bool,

    pub uids: Vec<Range>,
    pub gids: Vec<Range>,

    pub environ_file: Option<PathBuf>,
    pub environ: BTreeMap<String, String>,
    pub resolv_conf_path: Option<PathBuf>,
    pub hosts_file_path: Option<PathBuf>,
    pub volumes: BTreeMap<PathBuf, Volume>,
}

impl PartialEq for Container {
    fn eq(&self, _other: &Container) -> bool { false }
}


pub fn container_validator<'a>() -> V::Structure<'a> {
    V::Structure::new()
    .member("setup", V::Sequence::new(builder_validator()))
    .member("image_cache_url", V::Scalar::new().optional())
    .member("auto_clean", V::Scalar::new().default(false))
    .member("environ", V::Mapping::new(V::Scalar::new(), V::Scalar::new()))
    .member("environ_file", V::Scalar::new().optional())
    .member("resolv_conf_path",
        V::Directory::new()  // Well, should be file
        .is_absolute(true).optional()
        .default("/etc/resolv.conf"))
    .member("hosts_file_path",
        V::Directory::new()  // Well, should be file
        .is_absolute(true).optional()
        .default("/etc/hosts"))
    .member("uids", V::Sequence::new(V::Scalar::new()))
    .member("gids", V::Sequence::new(V::Scalar::new()))
    .member("volumes", V::Mapping::new(
        V::Directory::new().is_absolute(true),
        volume_validator()))
}
