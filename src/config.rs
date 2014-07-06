use std::io::File;

use collections::treemap::TreeMap;
use serialize::json::ToJson;

use quire::parse;
use N = quire::parser;
use J = serialize::json;


pub struct Command {
    pub run: Option<String>,
    pub container: Option<String>,
    pub accepts_arguments: bool,
}

pub struct Variant {
    pub default_variant: String,
    pub options: Vec<String>,
}

pub struct Container {
    pub default_command: String,
    pub settings: TreeMap<String, String>,
}

pub struct Config {
    pub commands: TreeMap<String, Command>,
    pub variants: TreeMap<String, Variant>,
    pub containers: TreeMap<String, Container>,
}


fn find_config_path(workdir: &Path) -> Option<Path> {
    let mut dir = workdir.clone();
    loop {
        if dir.join("vagga.yaml").exists() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn get_string(json: &J::Json, key: &'static str) -> Option<String> {
    return match json {
        &J::Object(ref dict) => match dict.find(&key.to_string()) {
            Some(&J::String(ref val)) => Some(val.clone()),
            _ => None,
        },
        _ => None,
    }
}

pub fn find_config(workdir: Path) -> Result<(Config, Path), String>{
    let cfg_dir = match find_config_path(&workdir) {
        Some(path) => path,
        None => return Err(format!(
            "Config not found in path {}", workdir.display())),
    };
    let filename = cfg_dir.join("vagga.yaml");
    let fname = filename.display();
    let data = match File::open(&filename).read_to_str() {
        Ok(data) => data,
        Err(e) => return Err(format!("{}: {}", fname, e)),
    };
    let json = match parse(data.as_slice(), |doc| {
        return doc.to_json();
    }) {
        Ok(json) => json,
        Err(e) => return Err(format!("{}: {}", fname, e)),
    };

    let mut config = Config {
        commands: TreeMap::new(),
        variants: TreeMap::new(),
        containers: TreeMap::new(),
    };

    let root = match json {
        J::Object(val) => val,
        _ => return Err(format!("{}: root node must be mapping", fname)),
    };

    match root.find(&"commands".to_string()) {
    Some(&J::Object(ref commands)) => {
        for (name, cmd) in commands.iter() {
            let run = match cmd.find(&"run".to_string()) {
                Some(&J::String(ref val)) => Some(val),
                Some(_) => return Err(format!(
                    "{}: The \"run\" value must be string", fname)),
                None => None,
            };
            let cmd = Command {
                run: get_string(cmd, "run"),
                container: get_string(cmd, "container"),
                accepts_arguments: true,
            };
            config.commands.insert(name.clone(), cmd);
        }
    }
    Some(_) => return Err(format!(
        "{}: commands key must be mapping", fname)),
    None => {}
    }

    return Ok((config, cfg_dir));
}
