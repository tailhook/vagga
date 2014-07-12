use std::io::File;

use collections::treemap::TreeMap;
use serialize::json::ToJson;

use quire::parse;
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
    pub default_command: Option<String>,
    pub wrapper_script: Option<String>,
    pub builder: String,
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

fn get_dict(json: &J::Json, key: &'static str) -> TreeMap<String, String> {
    let mut res = TreeMap::new();
    let dict = match json {
        &J::Object(ref dict) => match dict.find(&key.to_string()) {
            Some(&J::Object(ref val)) => val,
            _ => return res,
        },
        _ => return res,
    };

    for (k, v) in dict.iter() {
        match v {
            &J::String(ref val) => {
                res.insert(k.clone(), val.clone());
            }
            _ => continue,  // TODO(tailhook) assert maybe?
        }
    }

    return res;
}

pub fn find_config(workdir: &Path) -> Result<(Config, Path), String>{
    let cfg_dir = match find_config_path(workdir) {
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
            for (name, jcmd) in commands.iter() {
                let cmd = Command {
                    run: get_string(jcmd, "run"),
                    container: get_string(jcmd, "container"),
                    accepts_arguments: true,  // TODO(tailhook)
                };
                config.commands.insert(name.clone(), cmd);
            }
        }
        Some(_) => return Err(format!(
            "{}: commands key must be mapping", fname)),
        None => {}
    }

    match root.find(&"containers".to_string()) {
        Some(&J::Object(ref containers)) => {
            for (name, jcont) in containers.iter() {
                let cont = Container {
                    default_command: get_string(jcont, "default-command"),
                    wrapper_script: get_string(jcont, "wrapper-script"),
                    builder: get_string(jcont, "builder")
                             .unwrap_or("nix".to_string()),
                    settings: get_dict(jcont, "settings"),
                };
                config.containers.insert(name.clone(), cont);
            }
        }
        Some(_) => return Err(format!(
            "{}: containers key must be mapping", fname)),
        None => {}
    }

    return Ok((config, cfg_dir));
}
