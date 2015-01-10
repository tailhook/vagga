use std::default::Default;
use std::collections::treemap::TreeMap;

use quire::validate as V;

use super::builders::{Builder, builder_validator};
use super::Range;

#[deriving(Decodable, Clone, PartialEq, Eq)]
pub enum Volume {
    Tmpfs(TmpfsInfo),
    VaggaBin,
}

#[deriving(Decodable, Clone, PartialEq, Eq)]
pub struct TmpfsInfo {
    pub size: uint,
    pub mode: u32,
}

#[deriving(Decodable, Clone)]
pub struct Container {
    pub setup: Vec<Builder>,

    pub uids: Vec<Range>,
    pub gids: Vec<Range>,

    pub environ_file: Option<Path>,
    pub environ: TreeMap<String, String>,
    pub resolv_conf: Option<Path>,
    pub volumes: TreeMap<Path, Volume>,
}

impl PartialEq for Container {
    fn eq(&self, _other: &Container) -> bool { false }
}

pub fn volume_validator<'a>() -> Box<V::Validator + 'a> {
    return box V::Enum { options: vec!(
        ("Tmpfs".to_string(),  box V::Structure { members: vec!(
            ("size".to_string(),  box V::Numeric {
                min: Some(0u),
                default: Some(100*1024*1024),
                .. Default::default()} as Box<V::Validator>),
            ("mode".to_string(),  box V::Numeric {
                min: Some(0u),
                max: Some(0o1777u),
                default: Some(0o766),
                .. Default::default()} as Box<V::Validator>),
            ),.. Default::default()} as Box<V::Validator>),
        ("VaggaBin".to_string(),  box V::Nothing),
        ), .. Default::default()} as Box<V::Validator>;
}

pub fn container_validator<'a>() -> Box<V::Validator + 'a> {
    return box V::Structure { members: vec!(
        ("setup".to_string(), box V::Sequence {
            element: builder_validator(),
            .. Default::default()} as Box<V::Validator>),
        ("environ".to_string(), box V::Mapping {
            key_element: box V::Scalar {
                .. Default::default()} as Box<V::Validator>,
            value_element: box V::Scalar {
                .. Default::default()} as Box<V::Validator>,
            .. Default::default()} as Box<V::Validator>),
        ("environ_file".to_string(), box V::Scalar {
            optional: true,
            .. Default::default()} as Box<V::Validator>),
        ("uids".to_string(), box V::Sequence {
            element: box V::Scalar {
                .. Default::default()} as Box<V::Validator>,
            .. Default::default()} as Box<V::Validator>),
        ("gids".to_string(), box V::Sequence {
            element: box V::Scalar {
                .. Default::default()} as Box<V::Validator>,
            .. Default::default()} as Box<V::Validator>),
        ("volumes".to_string(), box V::Mapping {
            key_element: box V::Directory {
                absolute: Some(true),
                .. Default::default()} as Box<V::Validator>,
            value_element: volume_validator(),
            .. Default::default()} as Box<V::Validator>),
        ), .. Default::default()} as Box<V::Validator>;
}

