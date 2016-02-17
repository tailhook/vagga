use std::collections::HashSet;

use config::{Config};


#[derive(PartialEq, Eq, Hash)]
struct Option<'a> {
    names: &'a [&'a str],
    has_args: bool,
    single: bool,
}

struct BuiltinCommand<'a> {
    name: &'a str,
    accept_container: bool,
    options: &'a [&'a Option<'a>],
}

/**

Transition table:
             _______________________
 +——————————|                       |
 |  +———————| GlobalOptionOrCommand |<———+
 |  |  +————|_______________________|    |
 |  |  |                                 |
 |  |  |        _________________        |
 |  |  |       |                 |       |
 |  |  +——————>| GlobalOptionArg |———————+
 |  |          |_________________|
 |  |
 |  |       __________________________
 |  +—————>|                          |
 |  +——————| CommandOptionOrContainer |<——+
 |  |  +———|__________________________|   |
 |  |  |                                  |
 |  |  |        __________________        |
 |  |  |       |                  |       |
 |  |  +——————>| CommandOptionArg |———————+
 |  |          |__________________|
 |  |
 |  |             ____________
 |  +———————————>|            |
 |               | CommandArg |
 +——————————————>|____________|

*/
enum States<'a> {
    GlobalOptionOrCommand,
    GlobalOptionArg(&'a Option<'a>),
    CommandOptionOrContainer(&'a BuiltinCommand<'a>),
    CommandOptionArg(&'a BuiltinCommand<'a>, &'a Option<'a>),
    CommandArg,
}

struct Completion<'a> {
    commands: &'a Vec<&'a String>,
    containers: &'a Vec<&'a String>,
    state: States<'a>,
    single_global_options: HashSet<&'a Option<'a>>,
    single_command_options: HashSet<&'a Option<'a>>,
}

impl<'a> Completion<'a> {
    pub fn new(commands: &'a Vec<&'a String>, containers: &'a Vec<&'a String>) -> Completion<'a> {
        Completion {
            commands: commands,
            containers: containers,
            state: States::GlobalOptionOrCommand,
            single_global_options: HashSet::new(),
            single_command_options: HashSet::new(),
        }
    }

    pub fn trans(&mut self, arg: &str) {
        match self.state {
            States::GlobalOptionOrCommand => {
                for cmd in BUILTIN_COMMANDS {
                    if cmd.name == arg {
                        self.state = States::CommandOptionOrContainer(cmd);
                        return;
                    }
                }
                for &cmd in self.commands {
                    if arg == cmd {
                        self.state = States::CommandArg;
                        return;
                    }
                }
                for opt in GLOBAL_OPTIONS {
                    for &opt_name in opt.names {
                        if arg == opt_name {
                            if opt.has_args {
                                self.state = States::GlobalOptionArg(opt);
                            }
                            if opt.single {
                                self.single_global_options.insert(opt);
                            }
                            return;
                        }
                    }
                }
            },
            States::GlobalOptionArg(_) => {
                self.state = States::GlobalOptionOrCommand;
            },
            States::CommandOptionOrContainer(cmd) => {
                for cmd_opt in cmd.options {
                    for &opt_name in cmd_opt.names {
                        if arg == opt_name {
                            if cmd_opt.has_args {
                                self.state = States::CommandOptionArg(cmd, cmd_opt);
                            }
                            if cmd_opt.single {
                                self.single_command_options.insert(cmd_opt);
                            }
                            return;
                        }
                    }
                }
                for &container_name in self.containers {
                    if arg == container_name {
                        self.state = States::CommandArg;
                        return;
                    }
                }
                self.state = States::CommandArg;
            },
            States::CommandOptionArg(cmd, _) => {
                self.state = States::CommandOptionOrContainer(cmd);
            },
            States::CommandArg => {},
        }
    }
    
    pub fn complete(&self, cur: &str) -> Vec<&str> {
        let mut completions: Vec<&str> = Vec::new();
        match self.state {
            States::GlobalOptionOrCommand => {
                completions.extend(self.commands.iter().map(|c| &c[..]));
                if cur.starts_with("_") {
                    completions.extend(BUILTIN_COMMANDS.iter().map(|c| c.name));
                }
                for opt in GLOBAL_OPTIONS {
                    if !self.single_global_options.contains(opt) {
                        completions.extend(opt.names);
                    }
                }
            },
            States::CommandOptionOrContainer(cmd) => {
                if cmd.accept_container {
                    completions.extend(self.containers.iter().map(|c| &c[..]));
                }
                for opt in cmd.options {
                    if !self.single_command_options.contains(opt) {
                        completions.extend(opt.names);
                    }
                }
            },
            _ => {},
        }
        completions.retain(|c| c.starts_with(cur));
        return completions;
    }
}

const BUILTIN_COMMANDS: &'static [&'static BuiltinCommand<'static>] = &[
    &BuiltinCommand { 
        name: "_build",
        accept_container: true,
        options: &[
            &Option { names: &["--force"], has_args: false, single: true },
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
            &Option { names: &["--tmp", "--tmp-folders"], has_args: false, single: true },
            &Option { names: &["--old", "--old-containers"], has_args: false, single: true },
            &Option { names: &["--unused"], has_args: false, single: true },
            &Option { names: &["--transient"], has_args: false, single: true },
            &Option { names: &["--global"], has_args: false, single: true },
            &Option { names: &["-n", "--dry-run"], has_args: false, single: true },
        ]
    },
    &BuiltinCommand { 
        name: "_create_netns",
        accept_container: false,
        options: &[
            &Option { names: &["--dry-run"], has_args: false, single: true },
            &Option { names: &["--no-iptables"], has_args: false, single: true },
        ]
    },
    &BuiltinCommand { 
        name: "_destroy_netns",
        accept_container: false,
        options: &[
            &Option { names: &["--dry-run"], has_args: false, single: true },
            &Option { names: &["--no-iptables"], has_args: false, single: true },
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
            &Option { names: &["-f", "--file"], has_args: true, single: true },
        ]
    },
    &BuiltinCommand { 
        name: "_run",
        accept_container: true,
        options: &[
            &Option { names: &["-W", "--writable"], has_args: false, single: true },
        ]
    },
    &BuiltinCommand { 
        name: "_run_in_netns",
        accept_container: true,
        options: &[
            &Option { names: &["--pid"], has_args: true, single: true },
        ]
    },
    &BuiltinCommand { 
        name: "_version_hash",
        accept_container: true,
        options: &[
            &Option { names: &["-s", "--short"], has_args: false, single: true },
            &Option { names: &["-fd3"], has_args: false, single: true },
        ]
    },
];

const GLOBAL_OPTIONS: &'static [&'static Option<'static>] = &[
    &Option { names: &["-E", "--env", "--environ"], has_args: true, single: false },
    &Option { names: &["-e", "--use-env"], has_args: true, single: false },
    &Option { names: &["--ignore-owner-check"], has_args: false, single: true },
    &Option { names: &["--no-build"], has_args: false, single: true },
    &Option { names: &["--no-version-check"], has_args: false, single: true },
];


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

    let commands = config.commands.keys().collect::<Vec<_>>();
    let containers = config.containers.keys().collect::<Vec<_>>();
    let mut state = Completion::new(&commands, &containers);
    for arg in full_args {
        state.trans(arg);
    }
    for comp in state.complete(cur_arg) {
        println!("{}", comp);
    }

    Ok(0)
}
