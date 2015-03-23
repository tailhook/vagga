use std::default::Default;
use std::collections::BTreeMap;

use quire::validate as V;
use quire::ast::Ast as A;
use quire::ast::Ast;
use quire::ast::Tag::{NonSpecific};
use quire::ast::ScalarKind::{Plain};

type PortNumValidator = V::Numeric<u16>;

#[derive(Decodable, Clone, PartialEq, Eq, Copy)]
#[allow(non_camel_case_types)]
pub enum Pid1Mode {
    exec = 0,
    wait = 1,
    wait_all_children = 2,
}

#[derive(Decodable, Show, Clone, PartialEq, Eq, Copy)]
#[allow(non_camel_case_types)]
pub enum SuperviseMode {
    wait_all,
    stop_on_failure,
    restart,
}

#[derive(Decodable, Show, Clone, PartialEq, Eq, Copy)]
#[allow(non_camel_case_types)]
pub enum WriteMode {
    read_only,
    //transient_reflink_copy, // TODO(tailhook)
    transient_hard_link_copy,
}

#[derive(Decodable, Clone, PartialEq, Eq)]
pub struct Network {
    pub ip: String,
    pub hostname: Option<String>,
    pub ports: BTreeMap<u16, u16>,
}

#[derive(Decodable, Clone, PartialEq, Eq)]
pub struct CommandInfo {
    // Common for toplevel commands
    pub description: Option<String>,
    pub banner: Option<String>,
    pub banner_delay: Option<u32>,
    pub epilog: Option<String>,

    // Command
    pub network: Option<Network>, // Only for top-levels
    pub pid1mode: Pid1Mode,
    pub work_dir: Option<String>,
    pub container: String,
    pub accepts_arguments: Option<bool>,
    pub environ: BTreeMap<String, String>,
    pub inherit_environ: Vec<String>,
    pub write_mode: WriteMode,
    pub run: Vec<String>,
    pub user_id: i32,
    pub external_user_id: Option<i32>,
}

#[derive(Decodable, Clone, PartialEq, Eq)]
pub struct SuperviseInfo {
    // Common
    pub description: Option<String>,
    pub banner: Option<String>,
    pub banner_delay: Option<u32>,
    pub epilog: Option<String>,

    // Supervise
    pub mode: SuperviseMode,
    pub children: BTreeMap<String, ChildCommand>,
}

#[derive(Decodable, PartialEq, Eq, Clone)]
pub enum MainCommand {
    Command(CommandInfo),
    Supervise(SuperviseInfo),
}

impl MainCommand {
    pub fn description<'x>(&'x self) -> Option<&'x String> {
        match *self {
            MainCommand::Command(ref cmd) => cmd.description.as_ref(),
            MainCommand::Supervise(ref cmd) => cmd.description.as_ref(),
        }
    }
}

#[derive(Decodable, PartialEq, Eq, Clone)]
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


fn run_fields<'a>(network: bool) -> Vec<(String, Box<V::Validator + 'a>)> {
    let mut res = vec!(
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
        ("write_mode".to_string(), box V::Scalar {
            default: Some("read-only".to_string()),
            .. Default::default()} as Box<V::Validator>),
        ("run".to_string(), box V::Sequence {
            from_scalar: Some(shell_command as fn(Ast) -> Vec<Ast>),
            element: box V::Scalar {
                .. Default::default()} as Box<V::Validator>,
            .. Default::default()} as Box<V::Validator>),
        ("user_id".to_string(), box V::Numeric {
            min: Some(0u32),
            max: Some(1 << 30),
            default: Some(0u32),
            .. Default::default()} as Box<V::Validator>),
        ("external_user_id".to_string(), box V::Numeric {
            min: Some(0u32),
            max: Some(1 << 30),
            optional: true,
            .. Default::default()} as Box<V::Validator>),
    );
    if network {
        res.push(
            ("network".to_string(), box V::Structure {
                optional: true,
                members: vec!(
                ("ip".to_string(), box V::Scalar {
                    optional: true,
                    .. Default::default()} as Box<V::Validator>),
                ("hostname".to_string(), box V::Scalar {
                    optional: true,
                    .. Default::default()} as Box<V::Validator>),
                ("ports".to_string(), box V::Mapping {
                    key_element: box V::Numeric {
                        default: None::<u16>,
                        .. Default::default()} as Box<V::Validator>,
                    value_element: box V::Numeric {
                        default: None::<u16>,
                        .. Default::default()} as Box<V::Validator>,
                    .. Default::default()} as Box<V::Validator>),
                ),.. Default::default()} as Box<V::Validator>),
        );
    }
    return res;
}

fn command_fields<'a>() -> Vec<(String, Box<V::Validator + 'a>)> {
    return vec!(
        ("description".to_string(), box V::Scalar {
            optional: true,
            .. Default::default()} as Box<V::Validator>),
        ("banner".to_string(), box V::Scalar {
            optional: true,
            .. Default::default()} as Box<V::Validator>),
        ("banner_delay".to_string(), box V::Numeric {
            optional: true,
            min: Some(0u32),
            .. Default::default()} as Box<V::Validator>),
        ("epilog".to_string(), box V::Scalar {
            optional: true,
            .. Default::default()} as Box<V::Validator>),
        );
}

fn subcommand_validator<'a>() -> Box<V::Validator + 'a> {
    return box V::Enum { options: vec!(
        ("Command".to_string(), box V::Structure {
            members: run_fields(true),
            .. Default::default()} as Box<V::Validator>),
        ("BridgeCommand".to_string(), box V::Structure {
            members: run_fields(false),
            .. Default::default()} as Box<V::Validator>),
    ), .. Default::default()} as Box<V::Validator>;
}

pub fn command_validator<'a>() -> Box<V::Validator + 'a> {
    let mut command_members = vec!();
    command_members.extend(command_fields().into_iter());
    command_members.extend(run_fields(false).into_iter());

    let mut supervise_members = vec!(
        ("mode".to_string(), box V::Scalar {
            default: Some("stop-on-failure".to_string()),
            .. Default::default()} as Box<V::Validator>),
        ("children".to_string(), box V::Mapping {
            key_element: box V::Scalar {
                ..Default::default()} as Box<V::Validator>,
            value_element: subcommand_validator(),
            .. Default::default()} as Box<V::Validator>),
        );
    supervise_members.extend(command_fields().into_iter());

    return box V::Enum { options: vec!(
        ("Command".to_string(), box V::Structure {
            members: command_members,
            .. Default::default()} as Box<V::Validator>),
        ("Supervise".to_string(), box V::Structure {
            members: supervise_members,
            .. Default::default()} as Box<V::Validator>),
    ), .. Default::default()} as Box<V::Validator>;
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
