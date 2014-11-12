#![crate_name="config"]
#![crate_type="lib"]
#![feature(phase)]

extern crate serialize;
extern crate quire;
extern crate regex;
#[phase(plugin)] extern crate regex_macros;
extern crate libc;

use std::default::Default;
use std::from_str::FromStr;
use std::io::fs::PathExtensions;
use libc::uid_t;

use std::collections::treemap::TreeMap;
use serialize::{Decoder, Decodable};
use quire::parse_config;

use quire::validate as V;
use quire::ast as A;
pub use self::settings::Settings;

pub mod settings;


#[deriving(Clone, PartialEq, Eq)]
pub enum Pid1Mode {
    Exec = 0,
    Wait = 1,
    WaitAllChildren = 2,
}

impl<E, D:Decoder<E>> Decodable<D, E> for Pid1Mode {
    fn decode(d: &mut D) -> Result<Pid1Mode, E> {
        d.read_enum("Pid1Mode", |d| {
            d.read_enum_variant(["exec", "wait", "wait-all-children"], |_, i| {
                Ok(match i {
                    0 => Exec,
                    1 => Wait,
                    2 => WaitAllChildren,
                    _ => unreachable!(),
                })
            })
        })
    }
}

#[deriving(Show, Clone, PartialEq, Eq)]
pub enum SuperviseMode {
    WaitAll,
    StopOnFailure,
    Restart,
}

impl<E, D:Decoder<E>> Decodable<D, E> for SuperviseMode {
    fn decode(d: &mut D) -> Result<SuperviseMode, E> {
        d.read_enum("Pid1Mode", |d| {
            d.read_enum_variant(
                ["wait-all", "stop-on-failure", "restart"], |_, i| {
                Ok(match i {
                    0 => WaitAll,
                    1 => StopOnFailure,
                    2 => Restart,
                    _ => unreachable!(),
                })
            })
        })
    }
}

#[deriving(Show, Clone, PartialEq, Eq)]
pub enum WriteMode {
    ReadOnly,
    TransientHardLinkCopy,
}

impl<E, D:Decoder<E>> Decodable<D, E> for WriteMode {
    fn decode(d: &mut D) -> Result<WriteMode, E> {
        d.read_enum("Pid1Mode", |d| {
            d.read_enum_variant(
                ["read-only", "transient-hard-link-copy"], |_, i| {
                Ok(match i {
                    0 => ReadOnly,
                    1 => TransientHardLinkCopy,
                    _ => unreachable!(),
                })
            })
        })
    }
}

#[deriving(Clone)]
pub enum Executor {
    Shell(String),
    Plain(Vec<String>),
    Supervise(SuperviseMode, TreeMap<String, Command>),
}

#[deriving(Clone)]
pub struct Command {
    pub name: String,
    pub pid1mode: Pid1Mode,
    pub work_dir: Option<String>,
    pub container: Option<String>,
    pub accepts_arguments: bool,
    pub environ: TreeMap<String, String>,
    pub inherit_environ: Vec<String>,
    pub description: Option<String>,
    pub resolv_conf: bool,
    pub write_mode: WriteMode,
    pub banner: Option<String>,
    pub banner_delay: i64,
    pub epilog: Option<String>,
    pub execute: Executor,
}

#[deriving(Decodable)]
pub struct GenericCommand {
    pub pid1mode: Pid1Mode,
    pub work_dir: Option<String>,
    pub container: Option<String>,
    pub accepts_arguments: Option<bool>,
    pub environ: TreeMap<String, String>,
    pub inherit_environ: Vec<String>,
    pub description: Option<String>,
    pub resolv_conf: bool,
    pub write_mode: WriteMode,
    pub banner: Option<String>,
    pub banner_delay: i64,
    pub epilog: Option<String>,

    pub run: Option<String>,
    pub command: Vec<String>,
    pub supervise: TreeMap<String, GenericCommand>,
    pub supervise_mode: SuperviseMode,
}

fn one_opt<T>(opt: &Option<T>) -> int {
    return if opt.is_some() { 1 } else { 0 };
}

fn one_len<T:Collection>(coll: &T) -> int {
    return if coll.len() > 0 { 1 } else { 0 };
}

impl GenericCommand {
    fn to_command(self, name: String) -> Result<Command, String> {
        if one_opt(&self.run) +
            one_len(&self.command) + one_len(&self.supervise) != 1
        {
            return Err(format!("Expected exactly one of \
                `command` or `run` or `supervise` for command {}",
                name));
        }
        if self.container.is_none() && self.supervise.len() == 0 {
            return Err(format!("The `container` is required for command {}",
                               name));
        }
        let mut accepts_arguments = false;
        let executor = if self.run.is_some() {
            Shell(self.run.unwrap())
        } else if self.command.len() > 0 {
            accepts_arguments = true;
            Plain(self.command)
        } else if self.supervise.len() > 0 {
            let svcs = self.supervise;
            let mut new = TreeMap::new();
            for (name, gcmd) in svcs.into_iter() {
                let cmd = try!(gcmd.to_command(name.clone()));
                new.insert(name, cmd);
            }

            Supervise(self.supervise_mode, new)
        } else {
            unreachable!();
        };
        return Ok(Command {
            name: name,
            pid1mode: self.pid1mode,
            work_dir: self.work_dir,
            container: self.container,
            accepts_arguments:
                self.accepts_arguments.unwrap_or(accepts_arguments),
            environ: self.environ,
            inherit_environ: self.inherit_environ,
            description: self.description,
            resolv_conf: self.resolv_conf,
            write_mode: self.write_mode,
            banner: self.banner,
            banner_delay: self.banner_delay,
            epilog: self.epilog,
            execute: executor,
        });
    }
}

impl PartialEq for GenericCommand {
    fn eq(&self, _other: &GenericCommand) -> bool { false }
}

#[deriving(Decodable)]
pub struct Variant {
    pub default: Option<String>,
    pub options: Vec<String>,
}

impl PartialEq for Variant {
    fn eq(&self, _other: &Variant) -> bool { false }
}

#[deriving(Clone, Show)]
pub struct Range {
    pub start: uid_t,
    pub end: uid_t,
}

impl<E, D:Decoder<E>> Decodable<D, E> for Range {
    fn decode(d: &mut D) -> Result<Range, E> {
        match d.read_str() {
            Ok(val) => {
                let num:Option<uid_t> = FromStr::from_str(val.as_slice());
                match num {
                    Some(num) => return Ok(Range::new(num, num)),
                    None => {}
                }
                match regex!(r"^(\d+)-(\d+)$").captures(val.as_slice()) {
                    Some(caps) => {
                        return Ok(Range::new(
                            from_str(caps.at(1)).unwrap(),
                            from_str(caps.at(2)).unwrap()));
                    }
                    None => unimplemented!(),
                }
            }
            Err(e) => Err(e),
        }
    }
}

#[deriving(Decodable, PartialEq, Clone)]
pub struct Directory {
    pub mode: u32,
}

#[deriving(Decodable)]
pub struct Container {
    pub default_command: Option<Vec<String>>,
    pub command_wrapper: Option<Vec<String>>,
    pub shell: Vec<String>,
    pub builder: String,
    pub provision: Option<String>,
    pub parameters: TreeMap<String, String>,
    pub environ_file: Option<String>,
    pub environ: TreeMap<String, String>,
    pub ensure_dirs: TreeMap<String, Directory>,
    pub uids: Vec<Range>,
    pub gids: Vec<Range>,
    pub tmpfs_volumes: TreeMap<String, String>,
}

impl PartialEq for Container {
    fn eq(&self, _other: &Container) -> bool { false }
}


#[deriving(Decodable)]
pub struct TmpConfig {
    pub commands: TreeMap<String, GenericCommand>,
    pub variants: TreeMap<String, Variant>,
    pub containers: TreeMap<String, Container>,
}

pub struct Config {
    pub commands: TreeMap<String, Command>,
    pub variants: TreeMap<String, Variant>,
    pub containers: TreeMap<String, Container>,
}

fn scalar_command(ast: A::Ast) -> Vec<A::Ast> {
    match ast {
        A::Scalar(pos, _, style, value) => {
            return value.as_slice().words().map(|w|
                A::Scalar(pos.clone(), A::NonSpecific, style, w.to_string()))
                .collect();
        }
        _ => unreachable!(),
    }
}

fn command_validator<'a>(supports_supervise: bool) -> Box<V::Validator + 'a> {
    let mut members = vec!(
        ("pid1mode".to_string(), box V::Scalar {
            default: Some("wait".to_string()),
            .. Default::default()} as Box<V::Validator>),
        ("work_dir".to_string(), box V::Scalar {
            optional: true,
            .. Default::default()} as Box<V::Validator>),
        ("container".to_string(), box V::Scalar {
            optional: true,
            .. Default::default()} as Box<V::Validator>),
        ("accepts_arguments".to_string(), box V::Scalar {
            optional: true,
            .. Default::default()} as Box<V::Validator>),
        ("environ".to_string(), box V::Mapping {
            key_element: box V::Scalar {
                .. Default::default()} as Box<V::Validator>,
            value_element: box V::Scalar {
                .. Default::default()} as Box<V::Validator>,
            .. Default::default()} as Box<V::Validator>),
        ("inherit_environ".to_string(), box V::Sequence {
            element: box V::Scalar {
                .. Default::default()} as Box<V::Validator>,
            .. Default::default()} as Box<V::Validator>),
        ("description".to_string(), box V::Scalar {
            optional: true,
            .. Default::default()} as Box<V::Validator>),
        ("resolv_conf".to_string(), box V::Scalar {
            default: Some("true".to_string()),
            .. Default::default()} as Box<V::Validator>),
        ("write_mode".to_string(), box V::Scalar {
            default: Some("read-only".to_string()),
            .. Default::default()} as Box<V::Validator>),
        ("banner".to_string(), box V::Scalar {
            optional: true,
            .. Default::default()} as Box<V::Validator>),
        ("banner_delay".to_string(), box V::Numeric {
            min: Some(0),
            default: Some(0i64),
            .. Default::default()} as Box<V::Validator>),
        ("epilog".to_string(), box V::Scalar {
            optional: true,
            .. Default::default()} as Box<V::Validator>),
        ("run".to_string(), box V::Scalar {
            optional: true,
            .. Default::default()} as Box<V::Validator>),
        ("command".to_string(), box V::Sequence {
            from_scalar: Some(scalar_command),
            element: box V::Scalar {
                .. Default::default()} as Box<V::Validator>,
            .. Default::default()} as Box<V::Validator>),
    );
    if supports_supervise {
        members.push(
            ("supervise".to_string(), box V::Mapping {
                key_element: box V::Scalar {
                    .. Default::default()} as Box<V::Validator>,
                value_element: command_validator(false),
                .. Default::default()} as Box<V::Validator>));
    }
    //  This should not be necessary, but since we don't make the field
    //  optional, we need to have it filled with default value when absent
    members.push(
        ("supervise_mode".to_string(), box V::Scalar {
            default: Some("stop-on-failure".to_string()),
            .. Default::default()} as Box<V::Validator>));
    return box V::Structure { members: members,
        .. Default::default()} as Box<V::Validator>;
}

fn container_validator<'a>() -> Box<V::Validator + 'a> {
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
        ("builder".to_string(), box V::Scalar {
            .. Default::default()} as Box<V::Validator>),
        ("provision".to_string(), box V::Scalar {
            optional: true,
            .. Default::default()} as Box<V::Validator>),
        ("parameters".to_string(), box V::Mapping {
            key_element: box V::Scalar {
                .. Default::default()} as Box<V::Validator>,
            value_element: box V::Scalar {
                .. Default::default()} as Box<V::Validator>,
            .. Default::default()} as Box<V::Validator>),
        ("ensure_dirs".to_string(), box V::Mapping {
            key_element: box V::Scalar {
                .. Default::default()} as Box<V::Validator>,
            value_element: box V::Structure { members: vec!(
                ("mode".to_string(), box V::Numeric {
                    min: Some(0),
                    max: Some(0o1777),
                    default: Some(0o755u32),
                    .. Default::default()} as Box<V::Validator>),
                // TODO(tailhook) owner and group
                // ("owner".to_string(), box V::Numeric {
                //     min: Some(0),
                //     max: Some(65534),
                //     default: Some(0),
                //     .. Default::default()} as Box<V::Validator>),
                // ("group".to_string(), box V::Numeric {
                //     min: Some(0),
                //     max: Some(65534),
                //     default: Some(0),
                //     .. Default::default()} as Box<V::Validator>),
                ), .. Default::default()} as Box<V::Validator>,
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

fn variant_validator<'a>() -> Box<V::Validator + 'a> {
    return box V::Structure { members: vec!(
        ), .. Default::default()} as Box<V::Validator>;
}

pub fn config_validator<'a>() -> Box<V::Validator + 'a> {
    return box V::Structure { members: vec!(
        ("containers".to_string(), box V::Mapping {
            key_element: box V::Scalar {
                .. Default::default()} as Box<V::Validator>,
            value_element: container_validator(),
            .. Default::default()} as Box<V::Validator>),
        ("commands".to_string(), box V::Mapping {
            key_element: box V::Scalar {
                .. Default::default()} as Box<V::Validator>,
            value_element: command_validator(true),
            .. Default::default()} as Box<V::Validator>),
        ("variants".to_string(), box V::Mapping {
            key_element: box V::Scalar {
                .. Default::default()} as Box<V::Validator>,
            value_element: variant_validator(),
            .. Default::default()} as Box<V::Validator>),
    ), .. Default::default()} as Box<V::Validator>;
}

impl Range {
    pub fn new(start: uid_t, end: uid_t) -> Range {
        return Range { start: start, end: end };
    }
    pub fn len(&self) -> uid_t {
        return self.end - self.start + 1;
    }
    pub fn shift(&self, val: uid_t) -> Range {
        assert!(self.end - self.start + 1 >= val);
        return Range::new(self.start + val, self.end);
    }
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

pub fn find_config(work_dir: &Path) -> Result<(Config, Path), String>{
    let (cfg_dir, filename) = match find_config_path(work_dir) {
        Some(pair) => pair,
        None => return Err(format!(
            "Config not found in path {}", work_dir.display())),
    };
    let mut tmp: TmpConfig = match parse_config(
        &filename, &*config_validator(), Default::default())
    {
        Ok(cfg) => cfg,
        Err(e) => {
            return Err(format!("Config {} cannot be read: {}",
                filename.display(), e));
        }
    };
    for (_, cont) in tmp.containers.iter_mut() {
        if cont.shell.len() == 0 {
            cont.shell.push("/bin/sh".to_string());
            cont.shell.push("-c".to_string());
        }
        if cont.tmpfs_volumes.len() == 0 {
            cont.tmpfs_volumes.insert(
                "/tmp".to_string(),
                "size=100m,mode=1777".to_string());
        }
    }
    let mut config = Config {
        commands: TreeMap::new(),
        containers: tmp.containers,
        variants: tmp.variants,
    };

    for (name, gcmd) in tmp.commands.into_iter() {
        let cmd = try!(gcmd.to_command(name.clone()));
        config.commands.insert(name, cmd);
    }

    return Ok((config, cfg_dir));
}
