use std::fs::File;
use std::path::{Path, PathBuf};

use rustc_serialize::json::Json;
use shaman::digest::Digest;

use version::error::Error;
use config::builders::ComposerDependencies;


pub fn hash(info: &composerdependencies, hash: &mut digest)
    -> Result<(), Error>
{
}

