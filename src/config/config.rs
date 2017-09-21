use std::fs::File;
use std::io::{stderr, Read, Write};
use std::mem;
use std::path::{PathBuf, Path, Component};
use std::rc::Rc;
use std::process::exit;

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
use super::version::{MinimumVagga, MinimumVaggaError};

#[derive(Deserialize)]
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

fn find_config_path(work_dir: &Path, verbose: bool)
    -> Option<(PathBuf, PathBuf)>
{
    let mut dir = work_dir.to_path_buf();
    loop {
        if verbose {
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

pub fn find_config(work_dir: &Path, verbose: bool)
    -> Result<(Config, PathBuf), String>
{
    let (cfg_dir, filename) = match find_config_path(work_dir, verbose) {
        Some(pair) => pair,
        None => return Err(format!(
            "Config not found in path {:?}", work_dir)),
    };
    assert!(cfg_dir.is_absolute());
    if verbose {
        info!("Found configuration file: {:?}", &filename);
    }
    let cfg = read_config(&filename, verbose)?;
    validate_config(&cfg)?;
    return Ok((cfg, cfg_dir));
}

pub fn find_config_or_exit(work_dir: &Path, verbose: bool)
    -> (Config, PathBuf)
{
    match find_config(work_dir, verbose) {
        Ok(pair) => pair,
        Err(e) => {
            writeln!(&mut stderr(),
                "Error parsing configuration file: {}. \
                 It usually happens either if configuration file was changed \
                 after vagga have been started or if permissions of the config \
                 file or some of the included files are wrong.", e).ok();
            exit(126);
        }
    }
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
            .map_err(|e| err.add_error(Error::open_error(&path, e))).ok()
            .and_then(|_| {
                parse_yaml(Rc::new(path.display().to_string()), &body,
                    |doc| { process_ast(&options, doc, err) },
                ).map_err(|e| err.add_error(e)).ok()
            })
            .unwrap_or_else(|| Ast::void(pos))
        }
    }
}

fn read_mixins(filename: &Path, mixins: &Vec<String>, dest: &mut Config,
    opt: &Options, verbose: bool)
    -> Result<(), String>
{
    for mixin in mixins.iter().rev() {
        let mixin_result: Result<(PathBuf, Config), _> =
            join_path(filename, mixin)
            .and_then(|path| {
                single_file(&path, opt, true)
                .map(move |c| (path, c))
            });
        match mixin_result {
            Ok((path, subcfg)) => {
                for (cname, cont) in subcfg.containers.into_iter() {
                    if !dest.containers.contains_key(&cname) {
                        info!("Container {:?} imported from {:?}",
                            cname, path);
                        dest.containers.insert(cname, cont);
                    }
                }
                for (cname, cmd) in subcfg.commands.into_iter() {
                    if !dest.commands.contains_key(&cname) {
                        info!("Command {:?} imported from {:?}", cname, path);
                        dest.commands.insert(cname, cmd);
                    }
                }
                read_mixins(&path, &subcfg.mixins, dest, opt, verbose)?;
            }
            Err(e) => {
                if verbose {
                    warn!("Skipping mixin because of error. Error: {}", e);
                }
            }
        }
    }
    Ok(())
}

fn single_file(filename: &Path, opt: &Options, is_mixin: bool)
    -> Result<Config, String>
{
    let filename = Rc::new(filename.to_path_buf());
    parse_config(&*filename, &config_validator(), &opt)
    .map_err(|e| {
        if let (true, Some(e)) = (
            is_mixin,
            e.errors()
                .find(|x| x.downcast_ref::<MinimumVaggaError>().is_some()))
        {
            format!("{}", e)
        } else {
            format!("{}", e)
        }
    })
    .map(|mut cfg: Config| {
        for (_, ref mut command) in &mut cfg.commands {
            command.set_source(filename.clone());
        }
        for (_, ref mut container) in &mut cfg.containers {
            container.source = Some(filename.clone());
        }
        cfg
    })
}

pub fn read_config(filename: &Path, verbose: bool) -> Result<Config, String> {
    let mut opt = Options::default();
    opt.allow_include(include_file);
    let mut config = single_file(filename, &opt, false)?;
    read_mixins(filename,
        &mem::replace(&mut config.mixins, Vec::new()),
        &mut config, &opt, verbose)?;

    // Is this a good place for such defaults?
    for (_, ref mut container) in config.containers.iter_mut() {
        if container.uids.len() == 0 {
            container.uids.push(Range::new(0, 65535));
        }
        if container.gids.len() == 0 {
            container.gids.push(Range::new(0, 65535));
        }
    }
    for (_, cmd) in config.commands.iter_mut() {
        match *cmd {
            MainCommand::CapsuleCommand(ref mut cmd) => {
                if cmd.uids.len() == 0 {
                    cmd.uids.push(Range::new(0, 65535));
                }
                if cmd.gids.len() == 0 {
                    cmd.gids.push(Range::new(0, 65535));
                }
            }
            _ => {}
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
