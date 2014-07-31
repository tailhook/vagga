use std::io::File;

use collections::treemap::TreeMap;
use serialize::json::ToJson;

use quire::parse;
use J = serialize::json;

use Pid1 = super::linux;
use super::yamlutil::{get_string, get_dict, get_list, get_command, get_bool};


#[deriving(Show)]
pub enum SuperviseMode {
    WaitAll,
    StopOnFailure,
    Restart,
}

pub enum Executor {
    Shell(String),
    Plain(Vec<String>),
    Supervise(SuperviseMode, TreeMap<String, Command>),
}


pub struct Command {
    pub pid1mode: Pid1::Pid1Mode,
    pub execute: Executor,
    pub work_dir: Option<String>,
    pub container: Option<String>,
    pub accepts_arguments: bool,
    pub environ: TreeMap<String, String>,
    pub inherit_environ: Vec<String>,
    pub description: Option<String>,
}

pub struct Variant {
    pub default: Option<String>,
    pub options: Vec<String>,
}

pub struct Container {
    pub default_command: Option<Vec<String>>,
    pub command_wrapper: Option<Vec<String>>,
    pub shell: Vec<String>,
    pub builder: String,
    pub parameters: TreeMap<String, String>,
    pub environ_file: Option<String>,
    pub environ: TreeMap<String, String>,
}

pub struct Config {
    pub commands: TreeMap<String, Command>,
    pub variants: TreeMap<String, Variant>,
    pub containers: TreeMap<String, Container>,
}


fn find_config_path(work_dir: &Path) -> Option<(Path, Path)> {
    let mut dir = work_dir.clone();
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

pub fn get_supervise(map: &TreeMap<String, J::Json>)
    -> Result<Option<(SuperviseMode, TreeMap<String, Command>)>, String>
{
    let mode = match map.find(&"supervise-mode".to_string()) {
        Some(&J::String(ref val)) => match val.as_slice() {
            "wait-all" => WaitAll,
            "stop-on-failure" => StopOnFailure,
            "restart" => Restart,
            _ => return Err(format!("The `supervise-mode` can be one of
                `wait-all`, `stop-on-failure` (default) or `restart`")),
        },
        None => StopOnFailure,
        _ => return Err(format!("The `supervise-mode` must be string")),
    };
    let mut commands = TreeMap::new();
    let ocommands = match map.find(&"supervise".to_string()) {
        None => return Ok(None),
        Some(&J::Object(ref map)) => map,
        _ => return Err(format!("The `supervise` must be mapping")),
    };
    for (name, jcmd) in ocommands.iter() {
        let cmd = try!(parse_command(name, jcmd));
        commands.insert(name.clone(), cmd);
    }
    return Ok(Some((mode, commands)));
}

fn one<T>(opt: &Option<T>) -> int {
    return if opt.is_some() { 1 } else { 0 };
}

fn parse_command(name: &String, jcmd: &J::Json) -> Result<Command, String> {
    let dcmd = match jcmd {
        &J::Object(ref dict) => dict,
        _ => return Err(format!(
            "Command {} must be mapping", name)),
    };

    let run = get_string(jcmd, "run");
    let command = get_command(jcmd, "command");
    let supervise = try!(get_supervise(*dcmd));
    if one(&run) + one(&command) + one(&supervise) != 1 {
        return Err(format!("Expected exactly one of \
            `command` or `run` or `supervise` for command {}",
            name));
    }
    let container = get_string(jcmd, "container");
    if container.is_none() && supervise.is_none() {
        return Err(format!("The `container` is required for command {}",
                           name));
    }
    let mut accepts_arguments = false;
    let executor = if run.is_some() {
        Shell(run.unwrap())
    } else if command.is_some() {
        accepts_arguments = true;
        Plain(command.unwrap())
    } else if supervise.is_some() {
        let (mode, svcs) = supervise.unwrap();
        Supervise(mode, svcs)
    } else {
        unreachable!();
    };
    let accepts_arguments = get_bool(jcmd, "accepts-arguments")
        //  By default accept arguments only by command
        //  because "sh -c" only uses arguments if there is
        //  $1, $2.. used in the expression
        .unwrap_or(accepts_arguments);
    return Ok(Command {
        pid1mode: match get_string(jcmd, "pid1mode") {
            Some(ref s) if s.as_slice() == "wait" => Pid1::Wait,
            Some(ref s) if s.as_slice() == "wait-any" => Pid1::WaitAny,
            Some(ref s) if s.as_slice() == "exec" => Pid1::Exec,
            None => Pid1::Wait,
            _ => return Err(format!("The pid1mode must be one of `wait`, \
                `wait-any` or `exec` for command {}", name)),
            },
        execute: executor,
        container: container,
        work_dir: get_string(jcmd, "work-dir"),
        accepts_arguments: accepts_arguments,
        environ: get_dict(jcmd, "environ"),
        inherit_environ: get_list(jcmd, "inherit-environ"),
        description: get_string(jcmd, "description"),
    });
}

pub fn find_config(work_dir: &Path) -> Result<(Config, Path), String>{
    let (cfg_dir, filename) = match find_config_path(work_dir) {
        Some(pair) => pair,
        None => return Err(format!(
            "Config not found in path {}", work_dir.display())),
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
                let cmd = try!(parse_command(name, jcmd));
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
                    default_command: get_command(jcont, "default-command"),
                    command_wrapper: get_command(jcont, "command-wrapper"),
                    shell: get_command(jcont, "shell").unwrap_or(
                        vec!("/bin/sh".to_string(), "-c".to_string())),
                    builder: get_string(jcont, "builder")
                             .unwrap_or("nix".to_string()),
                    parameters: get_dict(jcont, "parameters"),
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
