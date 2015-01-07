use std::default::Default;
use std::collections::TreeMap;

use quire::validate as V;
use quire::ast as A;

pub use self::main::MainCommand;
pub use self::child::ChildCommand;


#[deriving(Decodable, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum Pid1Mode {
    exec = 0,
    wait = 1,
    wait_all_children = 2,
}

#[deriving(Decodable, Show, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum SuperviseMode {
    wait_all,
    stop_on_failure,
    restart,
}

#[deriving(Decodable, Show, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum WriteMode {
    read_only,
    transient_reflink_copy,
    transient_hardlink_copy,
}

#[deriving(Decodable, Clone, PartialEq, Eq)]
pub struct Network {
    pub ip: String,
}

#[deriving(Decodable, Clone, PartialEq, Eq)]
pub struct CommandInfo {
    // Common
    pub description: Option<String>,
    pub banner: Option<String>,
    pub banner_delay: i64,
    pub epilog: Option<String>,

    // Command
    pub network: Option<Network>,
    pub pid1mode: Pid1Mode,
    pub work_dir: Option<String>,
    pub container: String,
    pub accepts_arguments: Option<bool>,
    pub environ: TreeMap<String, String>,
    pub inherit_environ: Vec<String>,
    pub write_mode: WriteMode,
    pub run: Vec<String>,
}

#[deriving(Decodable, Clone, PartialEq, Eq)]
pub struct ChildCommandInfo {
    // Command
    pub network: Option<Network>,
    pub pid1mode: Pid1Mode,
    pub work_dir: Option<String>,
    pub container: String,
    pub accepts_arguments: Option<bool>,
    pub environ: TreeMap<String, String>,
    pub inherit_environ: Vec<String>,
    pub write_mode: WriteMode,
    pub run: Vec<String>,
}

#[deriving(Decodable, Clone, PartialEq, Eq)]
pub struct SuperviseInfo {
    // Common
    pub description: Option<String>,
    pub banner: Option<String>,
    pub banner_delay: i64,
    pub epilog: Option<String>,

    // Supervise
    pub mode: SuperviseMode,
    pub children: TreeMap<String, ChildCommand>,
}

pub mod main {
    use super::{CommandInfo, SuperviseInfo};

    #[deriving(Decodable, PartialEq, Eq, Clone)]
    pub enum MainCommand {
        Command(CommandInfo),
        Supervise(SuperviseInfo),
    }

    impl MainCommand {
        pub fn description<'x>(&'x self) -> Option<&'x String> {
            match *self {
                Command(ref cmd) => cmd.description.as_ref(),
                Supervise(ref cmd) => cmd.description.as_ref(),
            }
        }
    }
}

pub mod child {
    use super::{ChildCommandInfo};

    #[deriving(Decodable, PartialEq, Eq, Clone)]
    pub enum ChildCommand {
        Command(ChildCommandInfo),
    }

    impl ChildCommand {
        pub fn get_container<'x>(&'x self) -> &String {
            match *self {
                Command(ref info) => &info.container,
            }
        }
    }
}

fn shell_command(ast: A::Ast) -> Vec<A::Ast> {
    match ast {
        A::Scalar(pos, _, style, value) => {
            return vec!(
                A::Scalar(pos.clone(), A::NonSpecific, A::Plain,
                          "/bin/sh".to_string()),
                A::Scalar(pos.clone(), A::NonSpecific, A::Plain,
                          "-c".to_string()),
                A::Scalar(pos.clone(), A::NonSpecific, style,
                          value),
                );
        }
        _ => unreachable!(),
    }
}


fn run_fields<'a>() -> Vec<(String, Box<V::Validator + 'a>)> {
    return vec!(
        ("network".to_string(), box V::Structure { members: vec!(
            ("ip".to_string(), box V::Scalar {
                optional: true,
                .. Default::default()} as Box<V::Validator>),
            ),.. Default::default()} as Box<V::Validator>),
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
            from_scalar: Some(shell_command),
            element: box V::Scalar {
                .. Default::default()} as Box<V::Validator>,
            .. Default::default()} as Box<V::Validator>),
        );
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
            min: Some(0),
            default: Some(0i64),
            .. Default::default()} as Box<V::Validator>),
        ("epilog".to_string(), box V::Scalar {
            optional: true,
            .. Default::default()} as Box<V::Validator>),
        );
}

fn subcommand_validator<'a>() -> Box<V::Validator + 'a> {
    return box V::Enum { options: vec!(
        ("Command".to_string(), box V::Structure {
            members: run_fields(),
            .. Default::default()} as Box<V::Validator>),
    ), .. Default::default()} as Box<V::Validator>;
}

pub fn command_validator<'a>() -> Box<V::Validator + 'a> {
    let mut command_members = vec!();
    command_members.extend(command_fields().into_iter());
    command_members.extend(run_fields().into_iter());

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
