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
pub struct TmpfsInfo {
    pub size: usize,
    pub mode: u32,
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
    pub volumes: BTreeMap<PathBuf, Volume>,
}

impl PartialEq for Container {
    fn eq(&self, _other: &Container) -> bool { false }
}

pub fn volume_validator<'a>() -> Box<V::Validator + 'a> {
    return Box::new(V::Enum { options: vec!(
        ("Tmpfs".to_string(),  Box::new(V::Structure { members: vec!(
            ("size".to_string(),  Box::new(V::Numeric {
                min: Some(0),
                default: Some(100*1024*1024),
                .. Default::default()}) as Box<V::Validator>),
            ("mode".to_string(),  Box::new(V::Numeric {
                min: Some(0),
                max: Some(0o1777),
                default: Some(0o766),
                .. Default::default()}) as Box<V::Validator>),
            ),.. Default::default()}) as Box<V::Validator>),
        ("VaggaBin".to_string(),  Box::new(V::Nothing)),
        ("BindRW".to_string(),  Box::new(V::Scalar {
             .. Default::default()})),
        ), .. Default::default()}) as Box<V::Validator>;
}

pub fn container_validator<'a>() -> Box<V::Validator + 'a> {
    return Box::new(V::Structure { members: vec!(
        ("setup".to_string(), Box::new(V::Sequence {
            element: builder_validator(),
            .. Default::default()}) as Box<V::Validator>),
        ("auto_clean".to_string(), Box::new(V::Scalar {
            default: Some("false".to_string()),
            .. Default::default()}) as Box<V::Validator>),
        ("environ".to_string(), Box::new(V::Mapping {
            key_element: Box::new(V::Scalar {
                .. Default::default()}) as Box<V::Validator>,
            value_element: Box::new(V::Scalar {
                .. Default::default()}) as Box<V::Validator>,
            .. Default::default()}) as Box<V::Validator>),
        ("environ_file".to_string(), Box::new(V::Scalar {
            optional: true,
            .. Default::default()}) as Box<V::Validator>),
        ("resolv_conf_path".to_string(), Box::new(V::Directory {
            absolute: Some(true),
            optional: true,
            default: Some(PathBuf::from("/etc/resolv.conf")),
            .. Default::default()}) as Box<V::Validator>),
        ("uids".to_string(), Box::new(V::Sequence {
            element: Box::new(V::Scalar {
                .. Default::default()}) as Box<V::Validator>,
            .. Default::default()}) as Box<V::Validator>),
        ("gids".to_string(), Box::new(V::Sequence {
            element: Box::new(V::Scalar {
                .. Default::default()}) as Box<V::Validator>,
            .. Default::default()}) as Box<V::Validator>),
        ("volumes".to_string(), Box::new(V::Mapping {
            key_element: Box::new(V::Directory {
                absolute: Some(true),
                .. Default::default()}) as Box<V::Validator>,
            value_element: volume_validator(),
            .. Default::default()}) as Box<V::Validator>),
        ), .. Default::default()}) as Box<V::Validator>;
}

