use std::os::homedir;
use std::io::stdio::{stdout, stderr};
use std::io::fs::{File};
use std::io::fs::PathExtensions;
use std::io::{Write, Truncate};
use std::default::Default;

use collections::treemap::TreeMap;

use argparse::{ArgumentParser, Store};
use quire::parse_config;
use quire::validate as V;
use quire::emit as Y;

use super::env::Environ;


#[deriving(Decodable)]
pub struct TmpSettings {
    pub variants: TreeMap<String, String>,
    pub version_check: Option<bool>,
}

pub struct Settings {
    pub variants: TreeMap<String, String>,
    pub version_check: bool,
}

impl Settings {
    pub fn new() -> Settings {
        return Settings {
            variants: TreeMap::new(),
            version_check: true,
        };
    }
    fn merge(&mut self, other: TmpSettings) {
        for (k, v) in other.variants.into_iter() {
            self.variants.insert(k, v);
        }
        other.version_check.map(|v| {
            self.version_check = v;
        });
    }
}

fn settings_validator<'a>() -> Box<V::Validator + 'a> {
    return box V::Structure { members: vec!(
        ("variants".to_string(), box V::Mapping {
            key_element: box V::Scalar {
                .. Default::default()} as Box<V::Validator>,
            value_element: box V::Scalar {
                .. Default::default()} as Box<V::Validator>,
            .. Default::default()} as Box<V::Validator>),
        ("version_check".to_string(), box V::Scalar {
            optional: true,
            default: None,
            .. Default::default()} as Box<V::Validator>),
    ), .. Default::default()} as Box<V::Validator>;
}

pub fn read_settings(env: &mut Environ) {
    let mut files = Vec::new();
    match homedir() {
        Some(home) => {
            files.push(home.join_many([".config/vagga/settings.yaml"]));
            files.push(home.join_many([".vagga/settings.yaml"]));
            files.push(home.join_many([".vagga.yaml"]));
        }
        None => {}
    }
    files.push(env.project_root.join(".vagga.settings.yaml"));
    files.push(env.local_vagga.join("settings.yaml"));

    let validator = settings_validator();
    for filename in files.iter() {
        if filename.exists() {
            match parse_config(filename, &*validator, Default::default()) {
                Ok(s) => {
                    env.settings.merge(s);
                }
                Err(e) => {
                    error!("Error in config {}: {}", filename.display(), e);
                }
            }
        }
    }
}

pub fn set_variant(env: &mut Environ, args: Vec<String>)
    -> Result<int, String>
{
    let mut key: String = "".to_string();
    let mut value: String = "".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut key)
            .add_argument("variant", box Store::<String>,
                "A name of the variant variable")
            .required();
        ap.refer(&mut value)
            .add_argument("value", box Store::<String>,
                "The value for the variant variable")
            .required();
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => return Ok(122),
        }
    }
    match env.config.variants.find(&key) {
        Some(_) => {},
        None => return Err(format!("No variable {} defined", key)),
    }

    let path = env.local_vagga.join("settings.yaml");
    if path.exists() {
    } else {
        let mut file = match File::open_mode(&path, Truncate, Write) {
            Ok(file) => file,
            Err(e) => return Err(format!("Error writing {}: {}",
                                         path.display(), e)),
        };
        let mut ctx = Y::Context::new(&mut file);
        let write = Ok(())
            .and(ctx.emit(Y::MapStart(None, None)))
            .and(ctx.emit(Y::Scalar(None, None, Y::Auto, "variants")))
            .and(ctx.emit(Y::MapStart(None, None)))
            .and(ctx.emit(Y::Scalar(None, None, Y::Auto, key.as_slice())))
            .and(ctx.emit(Y::Scalar(None, None, Y::Auto, value.as_slice())))
            .and(ctx.emit(Y::MapEnd))
            .and(ctx.emit(Y::MapEnd));
        return match write {
            Ok(()) => Ok(0),
            Err(e) => Err(format!("Error writing yaml: {}", e)),
        };
    }


    return Ok(0);
}
