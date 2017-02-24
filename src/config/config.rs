use std::io::{Read};
use std::rc::Rc;
use std::fs::File;
use std::path::{PathBuf, Path, Component};

use std::collections::BTreeMap;

use quire::{Options, Include, Error, ErrorCollector, Pos, parse_config};
use quire::validate as V;
use quire::{raw_parse as parse_yaml};
use quire::ast::{Ast, process as process_ast};

use super::containers;
use super::containers::Container;
use super::command::{MainCommand, command_validator};
use super::range::Range;
use super::validate::validate_config;
use super::version::MinimumVagga;

#[derive(RustcDecodable)]
pub struct Config {
    pub minimum_vagga: Option<String>,
    pub mixins: Vec<String>,
    pub commands: BTreeMap<String, MainCommand>,
    pub containers: BTreeMap<String, Container>,
}

impl Config {
    pub fn get_container(&self, name: &str) -> Result<&Container, String> {
        self.containers.get(name)
        .ok_or_else(|| format!("Container {:?} not found", name))
    }
}

pub fn config_validator<'a>() -> V::Structure<'a> {
    V::Structure::new()
    .member("minimum_vagga", MinimumVagga::new()
        .optional()
        .current_version(env!("VAGGA_VERSION").to_string()))
    .member("mixins", V::Sequence::new(V::Scalar::new()))
    .member("containers", V::Mapping::new(
        V::Scalar::new(),
        containers::container_validator()))
    .member("commands", V::Mapping::new(
        V::Scalar::new(),
        command_validator()))
}

fn find_config_path(work_dir: &PathBuf, show_warnings: bool)
    -> Option<(PathBuf, PathBuf)>
{
    let mut dir = work_dir.clone();
    loop {
        if show_warnings {
            maybe_print_typo_warning(&dir.join(".vagga"));
            maybe_print_typo_warning(&dir);
        }

        let fname = dir.join(".vagga/vagga.yaml");
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

pub fn find_config(work_dir: &PathBuf, show_warnings: bool)
    -> Result<(Config, PathBuf), String>
{
    let (cfg_dir, filename) = match find_config_path(
        work_dir, show_warnings)
    {
        Some(pair) => pair,
        None => return Err(format!(
            "Config not found in path {:?}", work_dir)),
    };
    assert!(cfg_dir.is_absolute());
    let cfg = read_config(&filename)?;
    validate_config(&cfg)?;
    return Ok((cfg, cfg_dir));
}

fn join_path<A, B>(base: A, relative: B) -> Result<PathBuf, String>
    where A: AsRef<Path>, B: AsRef<Path>,
{
    let mut path = PathBuf::from(base.as_ref());
    path.pop(); // pop original filename
    for component in relative.as_ref().components() {
        match component {
            Component::Normal(x) => path.push(x),
            _ => {
                return Err(format!("Only relative paths without parent \
                             directories can be included"));
            }
        }
    }
    return Ok(path);
}

fn include_file(pos: &Pos, include: &Include,
    err: &ErrorCollector, options: &Options)
    -> Ast
{
    match *include {
        Include::File { filename } => {
            let path = match join_path(&*pos.filename, &filename) {
                Ok(path) => path,
                Err(e) => {
                    err.add_error(Error::preprocess_error(pos, e));
                    return Ast::void(pos);
                }
            };

            debug!("{} Including {:?}", pos, path);

            let mut body = String::new();
            File::open(&path)
            .and_then(|mut f| f.read_to_string(&mut body))
            .map_err(|e| err.add_error(Error::OpenError(path.clone(), e))).ok()
            .and_then(|_| {
                parse_yaml(Rc::new(path.display().to_string()), &body,
                    |doc| { process_ast(&options, doc, err) },
                ).map_err(|e| err.add_error(e)).ok()
            })
            .unwrap_or_else(|| Ast::void(pos))
        }
    }
}

pub fn read_config(filename: &Path) -> Result<Config, String> {
    let mut opt = Options::default();
    opt.allow_include(include_file);
    let mut config: Config =
        parse_config(filename, &config_validator(), &opt)
        .map_err(|e| format!("{}", e))?;

    for mixin in &config.mixins {
        let mixin_result: Result<Config, _> =
            join_path(filename, mixin)
            .and_then(|path| {
                parse_config(path, &config_validator(), &opt)
                .map_err(|e| format!("{}", e))
            });
        match mixin_result {
            Ok(subcfg) => {
                // TODO(tailhook) recursively apply mixins
                for (cname, cont) in subcfg.containers.into_iter() {
                    // TODO(tailhook) what to do with conflicts?
                    config.containers.insert(cname, cont);
                }
                for (cname, cmd) in subcfg.commands.into_iter() {
                    // TODO(tailhook) what to do with conflicts?
                    config.commands.insert(cname, cmd);
                }
            }
            Err(e) => {
                warn!("Skipping mixin because of error. Error: {}", e);
            }
        }
    }

    for (_, ref mut container) in config.containers.iter_mut() {
        if container.uids.len() == 0 {
            container.uids.push(Range::new(0, 65535));
        }
        if container.gids.len() == 0 {
            container.gids.push(Range::new(0, 65535));
        }
    }
    return Ok(config);
}

fn maybe_print_typo_warning(dir: &Path) {
    if dir.join("vagga.yml").exists() {
        warn!("There is vagga.yml file in the {:?}, \
               possibly it is a typo. \
               Correct configuration file name is vagga.yaml",
            dir);
    }
}
