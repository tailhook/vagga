use std::collections::{BTreeMap, HashSet};

use config::Config;
use config::command::MainCommand;
use config::containers::Container;


#[derive(PartialEq, Eq, Hash)]
struct CommandOption<'a> {
    names: &'a [&'a str],
    has_args: bool,
    single: bool,
}

struct BuiltinCommand<'a> {
    name: &'a str,
    accept_container: bool,
    options: &'a [&'a CommandOption<'a>],
}

#[derive(PartialEq, Eq, Hash)]
struct SuperviseOption<'a> {
    names: &'a [&'a str],
    has_args: bool,
    single: bool,
    accept_children: bool,
}


const GLOBAL_OPTIONS: &'static [&'static CommandOption<'static>] = &[
    &CommandOption { names: &["-V", "--version"], has_args: false, single: true },
    &CommandOption { names: &["-E", "--env", "--environ"], has_args: true, single: false },
    &CommandOption { names: &["-e", "--use-env"], has_args: true, single: false },
    &CommandOption { names: &["--ignore-owner-check"], has_args: false, single: true },
    &CommandOption { names: &["--no-build"], has_args: false, single: true },
    &CommandOption { names: &["--no-version-check"], has_args: false, single: true },
];

const SUPERVISE_OPTIONS: &'static [&'static SuperviseOption<'static>] = &[
    &SuperviseOption { names: &["--only"], has_args: true, single: true, accept_children: true },
    &SuperviseOption { names: &["--exclude"], has_args: true, single: true, accept_children: true },
    &SuperviseOption { names: &["--no-build"], has_args: false, single: true, accept_children: false },
    &SuperviseOption { names: &["--no-version-check"], has_args: false, single: true, accept_children: false },
];

const NO_BUILD: &'static CommandOption<'static> = &CommandOption {
    names: &["--no-build"], has_args: false, single: true
};

const NO_VERSION_CHECK: &'static CommandOption<'static> = &CommandOption {
    names: &["--no-version-check"], has_args: false, single: true
};

const BUILTIN_COMMANDS: &'static [&'static BuiltinCommand<'static>] = &[
    &BuiltinCommand {
        name: "_build",
        accept_container: true,
        options: &[
            &CommandOption { names: &["--force"], has_args: false, single: true },
        ]
    },
    &BuiltinCommand {
        name: "_build_shell",
        accept_container: false,
        options: &[]
    },
    &BuiltinCommand {
        name: "_clean",
        accept_container: false,
        options: &[
            &CommandOption { names: &["--tmp", "--tmp-folders"], has_args: false, single: true },
            &CommandOption { names: &["--old", "--old-containers"], has_args: false, single: true },
            &CommandOption { names: &["--unused"], has_args: false, single: true },
            &CommandOption { names: &["--transient"], has_args: false, single: true },
            &CommandOption { names: &["--global"], has_args: false, single: true },
            &CommandOption { names: &["-n", "--dry-run"], has_args: false, single: true },
        ]
    },
    &BuiltinCommand {
        name: "_create_netns",
        accept_container: false,
        options: &[
            &CommandOption { names: &["--dry-run"], has_args: false, single: true },
            &CommandOption { names: &["--no-iptables"], has_args: false, single: true },
        ]
    },
    &BuiltinCommand {
        name: "_destroy_netns",
        accept_container: false,
        options: &[
            &CommandOption { names: &["--dry-run"], has_args: false, single: true },
            &CommandOption { names: &["--no-iptables"], has_args: false, single: true },
        ]
    },
    &BuiltinCommand {
        name: "_init_storage_dir",
        accept_container: false,
        options: &[]
    },
    &BuiltinCommand {
        name: "_list",
        accept_container: false,
        options: &[]
    },
    &BuiltinCommand {
        name: "_pack_image",
        accept_container: true,
        options: &[
            &CommandOption { names: &["-f", "--file"], has_args: true, single: true },
            NO_BUILD,
            NO_VERSION_CHECK,
        ]
    },
    &BuiltinCommand {
        name: "_run",
        accept_container: true,
        options: &[
            &CommandOption { names: &["-W", "--writable"], has_args: false, single: true },
            NO_BUILD,
            NO_VERSION_CHECK,
        ]
    },
    &BuiltinCommand {
        name: "_run_in_netns",
        accept_container: true,
        options: &[
            &CommandOption { names: &["--pid"], has_args: true, single: true },
            NO_BUILD,
            NO_VERSION_CHECK,
        ]
    },
    &BuiltinCommand {
        name: "_version_hash",
        accept_container: true,
        options: &[
            &CommandOption { names: &["-s", "--short"], has_args: false, single: true },
            &CommandOption { names: &["-fd3"], has_args: false, single: true },
        ]
    },
];


/**

Transition table:

            ___________                     _________
           |           |——————————————————>|         |
  +————————| GlobalCmd |———————————+       | UserCmd |
  |  +—————|___________|——————+    |       |_________|
  |  |                        |    |
  |  |     ______________     |    |      ______________
  |  +———>|              |————+    +————>|              |
  |       | GlobalOption |               | SuperviseCmd |<—————+
  |  +————|______________|<———+    +—————|______________|      |
  |  |                        |    |                           |
  |  |    _________________   |    |     _________________     |
  |  |   |                 |  |    +———>|                 |————+
  |  +——>| GlobalOptionArg |——+         | SuperviseOption |
  |      |_________________|       +————|_________________|<———+
  |                                |                           |
  |        ____________            |    ____________________   |
  +——————>|            |           |   |                    |  |
  +———————| BuiltinCmd |<—————+    +——>| SuperviseOptionArg |——+
  |  +————|____________|      |        |____________________|
  |  |                        |
  |  |    _______________     |
  |  +——>|               |————+
  |      | BuiltinOption |
  |  +———|_______________|<———+
  |  |                        |
  |  |   __________________   |
  |  |  |                  |  |
  |  +—>| BuiltinOptionArg |——+
  |     |__________________|
  |
  |       ______________
  |      |              |
  +—————>| ContainerCmd |
         |______________|

*/
enum States<'a> {
    GlobalCmd,
    GlobalOption(&'a CommandOption<'a>),
    GlobalOptionArg(&'a CommandOption<'a>),
    UserCmd,
    SuperviseCmd(&'a str),
    SuperviseOption(&'a str, &'a SuperviseOption<'a>),
    SuperviseOptionArg(&'a str, &'a SuperviseOption<'a>),
    BuiltinCmd(&'a BuiltinCommand<'a>),
    BuiltinOption(&'a BuiltinCommand<'a>, &'a CommandOption<'a>),
    BuiltinOptionArg(&'a BuiltinCommand<'a>, &'a CommandOption<'a>),
    ContainerCmd,
}

struct CompletionState<'a> {
    commands: &'a BTreeMap<String, MainCommand>,
    containers: &'a BTreeMap<String, Container>,
    state: States<'a>,
    single_global_options: HashSet<&'a CommandOption<'a>>,
    single_command_options: HashSet<&'a CommandOption<'a>>,
    supervise_single_options: HashSet<&'a SuperviseOption<'a>>,
    supervise_chosen_children: HashSet<&'a str>,
}

impl<'a> CompletionState<'a> {
    pub fn new(
        commands: &'a BTreeMap<String, MainCommand>,
        containers: &'a BTreeMap<String, Container>
    ) -> CompletionState<'a> {

        CompletionState {
            commands: commands,
            containers: containers,
            state: States::GlobalCmd,
            single_global_options: HashSet::new(),
            single_command_options: HashSet::new(),
            supervise_single_options: HashSet::new(),
            supervise_chosen_children: HashSet::new(),
        }
    }

    pub fn trans(&mut self, arg: &'a str) {
        let mut next_state: Option<States> = None;
        {
            match self.state {
                States::GlobalCmd |
                States::GlobalOptionArg(_) => {
                    next_state = self.maybe_user_cmd(arg);
                    if let None = next_state {
                        next_state = self.maybe_global_option(arg);
                    }
                    if let None = next_state {
                        next_state = self.maybe_builtin_cmd(arg);
                    }
                },
                States::GlobalOption(opt) => {
                    if opt.has_args {
                        next_state = Some(States::GlobalOptionArg(opt));
                    } else {
                        next_state = self.maybe_user_cmd(arg);
                        if let None = next_state {
                            next_state = self.maybe_global_option(arg);
                        }
                        if let None = next_state {
                            next_state = self.maybe_builtin_cmd(arg);
                        }
                        if let None = next_state {
                            next_state = Some(States::GlobalCmd);
                        }
                    }
                },
                States::UserCmd => {},
                States::SuperviseCmd(cmd_name) |
                States::SuperviseOptionArg(cmd_name, _) => {
                    next_state = self.maybe_supervise_option(arg, cmd_name);
                },
                States::SuperviseOption(cmd_name, opt) => {
                    if opt.has_args {
                        next_state = Some(States::SuperviseOptionArg(cmd_name, opt));
                    } else {
                        next_state = self.maybe_supervise_option(arg, cmd_name);
                        if let None = next_state {
                            next_state = Some(States::SuperviseCmd(cmd_name));
                        }
                    }
                },
                States::BuiltinCmd(cmd) |
                States::BuiltinOptionArg(cmd, _) => {
                    next_state = self.maybe_builtin_option(arg, cmd);
                    if let None = next_state {
                        next_state = Some(States::ContainerCmd);
                    }
                },
                States::BuiltinOption(cmd, opt) => {
                    if opt.has_args {
                        next_state = Some(States::BuiltinOptionArg(cmd, opt));
                    } else {
                        next_state = self.maybe_builtin_option(arg, cmd);
                        if let None = next_state {
                            next_state = Some(States::BuiltinCmd(cmd));
                        }
                    }
                },
                States::ContainerCmd => {},
            }
        }

        if let Some(next_state) = next_state {
            match next_state {
                States::SuperviseOption(_, opt) if opt.single => {
                    self.supervise_single_options.insert(opt);
                },
                States::SuperviseOptionArg(cmd_name, opt) => {
                    if let Some(&MainCommand::Supervise(ref cmd_info)) = self.commands.get(cmd_name) {
                        if opt.accept_children {
                            if cmd_info.children.contains_key(arg) {
                                self.supervise_chosen_children.insert(arg);
                            }
                        }
                    }
                },
                States::GlobalOption(opt) if opt.single => {
                    self.single_global_options.insert(opt);
                },
                States::BuiltinOption(_, opt) if opt.single => {
                    self.single_command_options.insert(opt);
                },
                _ => {},
            }
            self.state = next_state;
        }
    }
    
    fn maybe_user_cmd(&self, arg: &'a str) -> Option<States<'a>> {
        for (cmd_name, user_cmd) in self.commands.iter() {
            if arg != cmd_name {
                continue;
            }
            match *user_cmd {
                MainCommand::Command(_) => {
                    return Some(States::UserCmd);
                },
                MainCommand::Supervise(_) => {
                    return Some(States::SuperviseCmd(cmd_name));
                },
            }
        }
        return None;
    }

    fn maybe_global_option(&self, arg: &'a str) -> Option<States<'a>> {
        for opt in GLOBAL_OPTIONS {
            for &opt_name in opt.names {
                if arg == opt_name {
                    return Some(States::GlobalOption(opt));
                }
            }
        }
        return None;
    }
    
    fn maybe_supervise_option(
        &self, arg: &'a str, cmd_name: &'a str
    )
        -> Option<States<'a>>
    {
        for opt in SUPERVISE_OPTIONS {
            for &opt_name in opt.names {
                if arg == opt_name {
                    return Some(States::SuperviseOption(cmd_name, opt));
                }
            }
        }
        return None;
    }
    
    fn maybe_builtin_cmd(&self, arg: &'a str) -> Option<States<'a>> {
        for cmd in BUILTIN_COMMANDS {
            if cmd.name == arg {
                return Some(States::BuiltinCmd(cmd));
            }
        }
        return None;
    }

    fn maybe_builtin_option(&self, arg: &'a str, cmd: &'a BuiltinCommand<'a>)
                            -> Option<States<'a>> {
        for cmd_opt in cmd.options {
            for &opt_name in cmd_opt.names {
                if arg == opt_name {
                    return Some(States::BuiltinOption(cmd, cmd_opt));
                }
            }
        }
        return None;
    }

    pub fn complete(&self, cur: &str) -> Vec<&str> {
        let mut completions: Vec<&str> = Vec::new();
        match self.state {
            States::GlobalCmd |
            States::GlobalOptionArg(_) => {
                completions.extend(self.complete_global(cur));
            },
            States::GlobalOption(opt) if !opt.has_args => {
                completions.extend(self.complete_global(cur));
            },
            States::SuperviseCmd(_) => {
                for opt in SUPERVISE_OPTIONS {
                    completions.extend(opt.names);
                }
            },
            States::SuperviseOption(cmd_name, opt) |
            States::SuperviseOptionArg(cmd_name, opt) => {
                completions.extend(
                    self.complete_supervise_options(cur, cmd_name, opt)
                );
            },
            States::BuiltinCmd(cmd) |
            States::BuiltinOptionArg(cmd, _) => {
                completions = self.complete_builtin(cur, cmd);
            },
            States::BuiltinOption(cmd, opt) if !opt.has_args => {
                completions = self.complete_builtin(cur, cmd);
            },
            _ => {},
        }
        completions.retain(|c| c.starts_with(cur));
        return completions;
    }

    fn complete_global(&self, cur: &str) -> Vec<&str> {
        let mut completions = Vec::new();
        completions.extend(self.commands.keys().map(|c| &c[..]));
        if cur.starts_with("_") {
            completions.extend(BUILTIN_COMMANDS.iter().map(|c| c.name));
        }
        if cur.starts_with("-") {
            for opt in GLOBAL_OPTIONS {
                if !self.single_global_options.contains(opt) {
                    completions.extend(opt.names);
                }
            }
        }
        return completions;
    }

    fn complete_supervise_options(
        &self, cur: &str, cmd_name: &'a str, opt: &SuperviseOption<'a>
    )
        -> Vec<&str>
    {
        let mut completions = Vec::new();
        if let Some(&MainCommand::Supervise(ref cmd_info)) = self.commands.get(cmd_name) {
            if opt.accept_children {
                for child in cmd_info.children.keys() {
                    let child_name = &child[..];
                    if !self.supervise_chosen_children.contains(child_name) {
                        completions.push(child_name);
                    }
                }
            }
        }
        if cur.starts_with("-") || !opt.has_args {
            for sv_opt in SUPERVISE_OPTIONS {
                if !self.supervise_single_options.contains(sv_opt) {
                    completions.extend(sv_opt.names);
                }
            }
        }
        return completions;
    }

    fn complete_builtin(&self, cur: &str, cmd: &BuiltinCommand<'a>) -> Vec<&str> {
        let mut completions = Vec::new();
        if cmd.accept_container {
            completions.extend(self.containers.keys().map(|c| &c[..]));
        }
        if cur.starts_with("-") {
            for opt in cmd.options {
                if !self.single_command_options.contains(opt) {
                    completions.extend(opt.names);
                }
            }
        }
        return completions;
    }
}


pub fn generate_completions(config: &Config, args: Vec<String>) -> Result<i32, String> {
    let default_cur_arg = "".to_string();
    let mut splitted_args = args.splitn(2, |a| a == "--");
    let full_args = match splitted_args.next() {
        Some(a) => a.iter().collect::<Vec<_>>(),
        None => vec!(),
    };
    let cur_arg = match splitted_args.next() {
        Some(a) => a.get(0).unwrap_or(&default_cur_arg),
        None => &default_cur_arg,
    };

    let mut state = CompletionState::new(&config.commands, &config.containers);
    for arg in full_args {
        state.trans(arg);
    }
    for comp in state.complete(cur_arg) {
        println!("{}", comp);
    }

    Ok(0)
}
