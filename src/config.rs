use std::io::File;

use collections::treemap::TreeMap;
use serialize::json::ToJson;

use quire::parse;
use J = serialize::json;

use super::yamlutil::{get_string, get_dict, get_list};


pub struct Command {
    pub run: Option<String>,
    pub container: Option<String>,
    pub accepts_arguments: bool,
    pub environ: TreeMap<String, String>,
    pub description: Option<String>,
}

pub struct Variant {
    pub default: Option<String>,
    pub options: Vec<String>,
}

pub struct Container {
    pub default_command: Option<String>,
    pub wrapper_script: Option<String>,
    pub builder: String,
    pub settings: TreeMap<String, String>,
    pub environ_file: Option<String>,
    pub environ: TreeMap<String, String>,
}

pub struct Config {
    pub commands: TreeMap<String, Command>,
    pub variants: TreeMap<String, Variant>,
    pub containers: TreeMap<String, Container>,
}


fn find_config_path(workdir: &Path) -> Option<(Path, Path)> {
    let mut dir = workdir.clone();
    loop {
        let fname = dir.join_many([".vagga", "vagga.yaml"]);
        if fname.exists() {
            return Some((dir, fname));
        }
        let fname = dir.join("vagga.yaml");
        if fname.exists() {
            return Some((dir, fname));
        }
        if !dir.pop() {
            return None;
        }
    }
}

pub fn find_config(workdir: &Path) -> Result<(Config, Path), String>{
    let (cfg_dir, filename) = match find_config_path(workdir) {
        Some(pair) => pair,
        None => return Err(format!(
            "Config not found in path {}", workdir.display())),
    };
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
                    environ: get_dict(jcmd, "environ"),
                    description: get_string(jcmd, "description"),
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
                    environ: get_dict(jcont, "environ"),
                    environ_file: get_string(jcont, "environ-file"),
                };
                config.containers.insert(name.clone(), cont);
            }
        }
        Some(_) => return Err(format!(
            "{}: containers key must be mapping", fname)),
        None => {}
    }

    match root.find(&"variants".to_string()) {
        Some(&J::Object(ref variants)) => {
            for (name, jvar) in variants.iter() {
                let var = Variant {
                    default: get_string(jvar, "default"),
                    options: get_list(jvar, "options"),
                };
                config.variants.insert(name.clone(), var);
            }
        }
        Some(_) => return Err(format!(
            "{}: variants key must be mapping", fname)),
        None => {}
    }

    return Ok((config, cfg_dir));
}
