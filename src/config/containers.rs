use std::default::Default;
use std::collections::treemap::TreeMap;

use quire::validate as V;

use super::builders::Builder;
use super::Range;


#[deriving(Decodable)]
pub struct Container {
    pub setup: Vec<Builder>,

    pub uids: Vec<Range>,
    pub gids: Vec<Range>,

    pub default_command: Option<Vec<String>>,
    pub command_wrapper: Option<Vec<String>>,
    pub shell: Vec<String>,
    pub environ_file: Option<String>,
    pub environ: TreeMap<String, String>,
    pub tmpfs_volumes: TreeMap<String, String>,  // volumes
}

impl PartialEq for Container {
    fn eq(&self, _other: &Container) -> bool { false }
}

pub fn container_validator<'a>() -> Box<V::Validator + 'a> {
    return box V::Structure { members: vec!(
        ("default_command".to_string(), box V::Scalar {
            optional: true,
            .. Default::default()} as Box<V::Validator>),
        ("command_wrapper".to_string(), box V::Scalar {
            optional: true,
            .. Default::default()} as Box<V::Validator>),
        ("shell".to_string(), box V::Sequence {
            element: box V::Scalar {
                .. Default::default()} as Box<V::Validator>,
            .. Default::default()} as Box<V::Validator>),
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
        ("tmpfs_volumes".to_string(), box V::Mapping {
            key_element: box V::Scalar {
                .. Default::default()} as Box<V::Validator>,
            value_element: box V::Scalar {
                .. Default::default()} as Box<V::Validator>,
            .. Default::default()} as Box<V::Validator>),
        ), .. Default::default()} as Box<V::Validator>;
}

