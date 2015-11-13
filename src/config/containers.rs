use std::default::Default;
use std::collections::BTreeMap;
use std::path::PathBuf;

use quire::validate as V;

use super::builders::{Builder, builder_validator};
use super::Range;

#[derive(RustcDecodable, Clone, PartialEq, Eq)]
pub enum Volume {
    Tmpfs(TmpfsInfo),
    BindRW(PathBuf),
    VaggaBin,
}

#[derive(RustcDecodable, Clone, PartialEq, Eq)]
pub struct Dir {
    pub mode: u32,
}

#[derive(RustcDecodable, Clone, PartialEq, Eq)]
pub struct TmpfsInfo {
    pub size: usize,
    pub mode: u32,
    pub subdirs: BTreeMap<PathBuf, Dir>,
}

#[derive(RustcDecodable, Clone)]
pub struct Container {
    pub setup: Vec<Builder>,
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

pub fn volume_validator<'x>() -> V::Enum<'x> {
    V::Enum::new()
        .option("Tmpfs",  V::Structure::new()
            .member("size",  V::Numeric::new()
                .min(0).default(100*1024*1024))
            .member("mode",  V::Numeric::new()
                .min(0).max(0o1777).default(0o766))
            .member("subdirs",
                V::Mapping::new(
                    V::Directory::new().is_absolute(false),
                    V::Structure::new()
                        .member("mode", V::Numeric::new()
                            .min(0).max(0o1777).default(0o766))
                )))
        .option("VaggaBin",  V::Nothing)
        .option("BindRW",  V::Scalar::new())
}

pub fn container_validator<'a>() -> V::Structure<'a> {
    V::Structure::new()
    .member("setup", V::Sequence::new(builder_validator()))
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

