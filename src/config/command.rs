use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;

use quire::validate as V;
use quire::ast::Ast as A;
use quire::ast::Ast;
use quire::ast::Tag::{NonSpecific};
use quire::ast::ScalarKind::{Plain};

use config::Range;
use super::volumes::{Volume, volume_validator};
use launcher::system::SystemInfo;

#[derive(Deserialize, Clone, PartialEq, Eq, Copy)]
#[allow(non_camel_case_types)]
pub enum Pid1Mode {
    exec = 0,
    wait = 1,
    wait_all_children = 2,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Copy)]
#[allow(non_camel_case_types)]
pub enum SuperviseMode {
    wait_all_successful,
    stop_on_failure,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Copy)]
#[allow(non_camel_case_types)]
pub enum WriteMode {
    read_only,
    //transient_reflink_copy, // TODO(tailhook)
    transient_hard_link_copy,
}

#[derive(Deserialize, Clone, PartialEq, Eq)]
pub struct Network {
    pub ip: String,
    pub hostname: Option<String>,
    pub ports: BTreeMap<u16, u16>,
}

#[derive(Deserialize, Clone, PartialEq, Eq)]
pub struct CommandInfo {
    // Common for toplevel commands
    pub source: Option<Rc<PathBuf>>,
    pub description: Option<String>,
    pub banner: Option<String>,
    pub banner_delay: Option<u32>,
    pub epilog: Option<String>,
    pub pass_tcp_socket: Option<String>,
    pub prerequisites: Vec<String>,
    pub options: Option<String>,  // Only for toplevel
    pub expect_inotify_limit: Option<usize>,
    pub symlink_name: Option<String>,

    // Command
    pub tags: Vec<String>,  // Only for supervise chidlren
    pub network: Option<Network>, // Only for supervise children
    pub pid1mode: Pid1Mode,
    pub work_dir: Option<String>,
    pub container: String,
    pub accepts_arguments: Option<bool>,
    pub environ: BTreeMap<String, String>,
    pub volumes: BTreeMap<PathBuf, Volume>,
    pub write_mode: WriteMode,
    pub run: Vec<String>,
    pub user_id: u32,
    pub external_user_id: Option<u32>,
    pub group_id: u32,
    pub supplementary_gids: Vec<u32>,
}

#[derive(Deserialize, Clone, PartialEq, Eq)]
pub struct CapsuleInfo {
    // Common for toplevel commands
    // TODO(tailhook) remove unuseful fields here
    pub source: Option<Rc<PathBuf>>,
    pub description: Option<String>,
    pub banner: Option<String>,
    pub banner_delay: Option<u32>,
    pub epilog: Option<String>,
    pub pass_tcp_socket: Option<String>,
    pub prerequisites: Vec<String>,
    pub options: Option<String>,  // Only for toplevel
    pub expect_inotify_limit: Option<usize>,
    pub symlink_name: Option<String>,

    // CapsuleCommand
    pub uids: Vec<Range>,
    pub gids: Vec<Range>,
    pub work_dir: Option<String>,
    pub accepts_arguments: Option<bool>,
    pub environ: BTreeMap<String, String>,
    pub run: Vec<String>,
}

#[derive(Deserialize, Clone, PartialEq, Eq)]
pub struct SuperviseInfo {
    // Common
    pub source: Option<Rc<PathBuf>>,
    pub description: Option<String>,
    pub banner: Option<String>,
    pub banner_delay: Option<u32>,
    pub epilog: Option<String>,
    pub prerequisites: Vec<String>,
    pub options: Option<String>,  // Only for toplevel
    pub expect_inotify_limit: Option<usize>,
    pub symlink_name: Option<String>,

    // Supervise
    pub mode: SuperviseMode,
    pub isolate_network: bool,
    pub kill_unresponsive_after: u32,
    pub children: BTreeMap<String, ChildCommand>,
}

#[derive(Deserialize, PartialEq, Eq, Clone)]
pub enum MainCommand {
    Command(CommandInfo),
    CapsuleCommand(CapsuleInfo),
    Supervise(SuperviseInfo),
}

pub struct LinkInfo<'a> {
    pub name: &'a str,
}


impl MainCommand {
    pub fn description<'x>(&'x self) -> Option<&'x String> {
        match *self {
            MainCommand::Command(ref cmd) => cmd.description.as_ref(),
            MainCommand::Supervise(ref cmd) => cmd.description.as_ref(),
            MainCommand::CapsuleCommand(ref cmd) => cmd.description.as_ref(),
        }
    }
    pub fn options<'x>(&'x self) -> Option<&'x String> {
        match *self {
            MainCommand::Command(ref cmd) => cmd.options.as_ref(),
            MainCommand::Supervise(ref cmd) => cmd.options.as_ref(),
            MainCommand::CapsuleCommand(ref cmd) => cmd.options.as_ref(),
        }
    }
    pub fn system<'x>(&'x self) -> SystemInfo {
        match *self {
            MainCommand::Command(ref cmd) => SystemInfo {
                expect_inotify_limit: cmd.expect_inotify_limit,
            },
            MainCommand::Supervise(ref cmd) => SystemInfo {
                expect_inotify_limit: cmd.expect_inotify_limit,
            },
            MainCommand::CapsuleCommand(ref cmd) => SystemInfo {
                expect_inotify_limit: cmd.expect_inotify_limit,
            },
        }
    }
    pub fn link(&self) -> Option<LinkInfo> {
        match *self {
            MainCommand::Command(ref cmd) => {
                cmd.symlink_name.as_ref().map(|name| {
                    LinkInfo {
                        name: name,
                    }
                })
            },
            MainCommand::CapsuleCommand(ref cmd) => {
                cmd.symlink_name.as_ref().map(|name| {
                    LinkInfo {
                        name: name,
                    }
                })
            },
            MainCommand::Supervise(ref cmd) => {
                cmd.symlink_name.as_ref().map(|name| {
                    LinkInfo {
                        name: name,
                    }
                })
            },
        }
    }
    pub fn set_source(&mut self, fname: Rc<PathBuf>) {
        match *self {
            MainCommand::Command(ref mut c) => c.source = Some(fname),
            MainCommand::CapsuleCommand(ref mut c) => c.source = Some(fname),
            MainCommand::Supervise(ref mut c) => c.source = Some(fname),
        }
    }
    pub fn source(&self) -> &Option<Rc<PathBuf>> {
        match *self {
            MainCommand::Command(ref c) => &c.source,
            MainCommand::CapsuleCommand(ref c) => &c.source,
            MainCommand::Supervise(ref c) => &c.source,
        }
    }
}

#[derive(Deserialize, PartialEq, Eq, Clone)]
pub enum ChildCommand {
    Command(CommandInfo),
    BridgeCommand(CommandInfo),
}

impl ChildCommand {
    pub fn get_container<'x>(&'x self) -> &String {
        match *self {
            ChildCommand::Command(ref info) => &info.container,
            ChildCommand::BridgeCommand(ref info) => &info.container,
        }
    }
    pub fn get_tags<'x>(&'x self) -> &Vec<String> {
        match *self {
            ChildCommand::Command(ref info) => &info.tags,
            ChildCommand::BridgeCommand(ref info) => &info.tags,
        }
    }
    pub fn get_volumes(&self) -> &BTreeMap<PathBuf, Volume> {
        match *self {
            ChildCommand::Command(ref info) => &info.volumes,
            ChildCommand::BridgeCommand(ref info) => &info.volumes,
        }
    }
    pub fn prerequisites<'x>(&'x self) -> &Vec<String> {
        match *self {
            ChildCommand::Command(ref cmd) => cmd.prerequisites.as_ref(),
            ChildCommand::BridgeCommand(ref cmd) => cmd.prerequisites.as_ref(),
        }
    }
    pub fn pass_socket(&self) -> Option<&String> {
        match *self {
            ChildCommand::Command(ref c) => c.pass_tcp_socket.as_ref(),
            ChildCommand::BridgeCommand(ref c) => c.pass_tcp_socket.as_ref(),
        }
    }
}

fn shell_command(ast: Ast) -> Vec<Ast> {
    match ast {
        A::Scalar(pos, _, style, value) => {
            return vec!(
                A::Scalar(pos.clone(), NonSpecific, Plain,
                          "/bin/sh".to_string()),
                A::Scalar(pos.clone(), NonSpecific, Plain,
                          "-c".to_string()),
                A::Scalar(pos.clone(), NonSpecific, style,
                          value),
                A::Scalar(pos.clone(), NonSpecific, Plain,
                          "--".to_string()),
                );
        }
        _ => unreachable!(),
    }
}


fn run_fields<'a>(cmd: V::Structure, network: bool) -> V::Structure {
    let mut cmd = cmd
        .member("pid1mode", V::Scalar::new().default("wait"))
        .member("work_dir", V::Scalar::new().optional())
        .member("container", V::Scalar::new())
        .member("accepts_arguments", V::Scalar::new().optional())
        .member("environ", V::Mapping::new(V::Scalar::new(), V::Scalar::new()))
        .member("volumes", V::Mapping::new(
            V::Directory::new().absolute(true),
            volume_validator()))
        .member("write_mode", V::Scalar::new().default("read-only"))
        .member("run", V::Sequence::new(V::Scalar::new())
            .parser(shell_command as fn(Ast) -> Vec<Ast>))
        .member("user_id", V::Numeric::new().min(0).max(1 << 30).default(0))
        .member("external_user_id",
            V::Numeric::new().min(0).max(1 << 30).optional())
        .member("group_id", V::Numeric::new()
            .min(0).max(1 << 30).default(0))
        .member("supplementary_gids",
            V::Sequence::new(V::Numeric::new().min(0).max(1 << 30)));
    if network {
        cmd = cmd
            .member("network", V::Structure::new().optional()
                .member("ip", V::Scalar::new().optional())
                .member("hostname", V::Scalar::new().optional())
                .member("ports", V::Mapping::new(
                    V::Numeric::new(),
                    V::Numeric::new())))
    }
    return cmd;
}

fn command_fields<'a>(mut cmd: V::Structure, toplevel: bool) -> V::Structure
{
    cmd = cmd
        .member("description", V::Scalar::new().optional())
        .member("banner", V::Scalar::new().optional())
        .member("banner_delay", V::Numeric::new().optional().min(0))
        .member("epilog", V::Scalar::new().optional())
        .member("tags", V::Sequence::new(V::Scalar::new()))
        .member("pass_tcp_socket", V::Scalar::new().optional())
        .member("expect_inotify_limit", V::Scalar::new().optional())
        .member("symlink_name", V::Scalar::new().optional())
        .member("prerequisites", V::Sequence::new(V::Scalar::new()));
    if toplevel {
        cmd = cmd.member("options", V::Scalar::new().optional());
    }
    return cmd;
}

fn subcommand_validator<'a>() -> V::Enum<'a> {
    V::Enum::new()
    .option("Command",
        run_fields(command_fields(V::Structure::new(), false), true))
    .option("BridgeCommand",
        run_fields(command_fields(V::Structure::new(), false), false))
}

pub fn command_validator<'a>() -> V::Enum<'a> {
    let cmd = run_fields(command_fields(V::Structure::new(), true), false);
    let sup = command_fields(V::Structure::new(), true);
    let caps = command_fields(V::Structure::new(), true);

    let sup = sup
        .member("mode", V::Scalar::new().default("stop-on-failure"))
        .member("isolate_network", V::Scalar::new().default(false))
        .member("children", V::Mapping::new(
            V::Scalar::new(),
            subcommand_validator()))
        .member("kill_unresponsive_after",
            V::Numeric::new().default(2).min(1).max(86400));

    let caps = caps
        .member("uids", V::Sequence::new(V::Scalar::new()))
        .member("gids", V::Sequence::new(V::Scalar::new()))
        .member("work_dir", V::Scalar::new().optional())
        .member("run", V::Sequence::new(V::Scalar::new())
            .parser(shell_command as fn(Ast) -> Vec<Ast>))
        .member("environ", V::Mapping::new(V::Scalar::new(), V::Scalar::new()))
        .member("accepts_arguments", V::Scalar::new().optional());

    return V::Enum::new()
        .option("Command", cmd)
        .option("CapsuleCommand", caps)
        .option("Supervise", sup);
}

pub trait Networking {
    fn network<'x>(&'x self) -> Option<&'x Network>;
}

impl Networking for ChildCommand {
    fn network<'x>(&'x self) -> Option<&'x Network> {
        match self {
            &ChildCommand::Command(ref cmd) => cmd.network.as_ref(),
            &ChildCommand::BridgeCommand(_) => None,
        }
    }
}
