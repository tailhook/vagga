use std::os::homedir;
use std::io::stdio::{stdout, stderr};
use std::io::fs::{File};
use std::io::{Write, Truncate};
use std::io::FileNotFound;

use collections::treemap::TreeMap;
use serialize::json::ToJson;

use argparse::{ArgumentParser, Store};
use quire::parse;
use Y = quire::emit;

use super::env::Environ;
use super::yamlutil::{get_dict, get_bool};


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

    for filename in files.iter() {
        debug!("Trying to open {}", filename.display());
        let data = match File::open(filename).read_to_str() {
            Ok(data) => data,
            Err(ref e) if e.kind == FileNotFound => { continue; }
            Err(e) => {
                warn!("{}: {}", filename.display(), e);
                continue;
            }
        };
        let json = match parse(data.as_slice(), |doc| {
            return doc.to_json();
        }) {
            Ok(json) => json,
            Err(e) => {
                warn!("{}: {}", filename.display(), e);
                continue;
            }
        };
        let dic = get_dict(&json, "variants");
        for (k, v) in dic.move_iter() {
            info!("{}: Setting {}={}", filename.display(), k, v);
            env.settings.variants.insert(k, v);
        }
        match get_bool(&json, "version-check") {
            Some(val) => { env.settings.version_check = val; }
            _ => {}
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
