use config::{Config};


#[derive(Debug)]
enum OptionNumArgs {
    Zero,
    Single,
    Multiple
}

#[derive(Debug)]
struct Option<'a> {
    names: &'a [&'a str],
    num_args: OptionNumArgs,
}

#[derive(Debug)]
struct BuiltinCommand<'a> {
    name: &'a str,
    accept_container: bool,
    options: &'a [&'a Option<'a>]
}

/**
   ----------------------------------              -------------------------------------
  |                                  \            |                                     \
  --> GlobalOptionOrCommand ---> GlobalOptionArg ---> CommandOptionOrContainer ---> CommandOptionArg ---> CommandArg
                  \                               |                   \                               |
                   -------------------------------                     -------------------------------
*/
#[derive(Debug)]
enum States<'a> {
    GlobalOptionOrCommand,
    // GlobalOptionOrArgOrCommand,
    GlobalOptionArg(&'a Option<'a>),
    CommandOptionOrContainer(&'a BuiltinCommand<'a>),
    // CommandOptionOrArgOrContainer,
    CommandOptionArg(&'a BuiltinCommand<'a>, &'a Option<'a>),
    CommandArg,
}

#[derive(Debug)]
struct Completion<'a> {
    commands: &'a Vec<&'a String>,
    containers: &'a Vec<&'a String>,
    state: States<'a>,
}

impl<'a> Completion<'a> {
    pub fn new(commands: &'a Vec<&'a String>, containers: &'a Vec<&'a String>) -> Completion<'a> {
        Completion {
            commands: commands,
            containers: containers,
            state: States::GlobalOptionOrCommand,
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
                            match opt.num_args {
                                OptionNumArgs::Single | OptionNumArgs::Multiple => {    
                                    self.state = States::GlobalOptionArg(opt);
                                    return;
                                },
                                _ => {},
                            }
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
                            match cmd_opt.num_args {
                                OptionNumArgs::Single | OptionNumArgs::Multiple => {
                                    self.state = States::CommandOptionArg(cmd, cmd_opt);
                                },
                                _ => {},
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
                    completions.extend(opt.names);
                }
            },
            States::CommandOptionOrContainer(cmd) => {
                if cmd.accept_container {
                    completions.extend(self.containers.iter().map(|c| &c[..]));
                }
                for opt in cmd.options {
                    completions.extend(opt.names);
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
            &Option { names: &["--force"], num_args: OptionNumArgs::Zero },
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
            &Option { names: &["--tmp", "--tmp-folders"], num_args: OptionNumArgs::Zero },
            &Option { names: &["--old", "--old-containers"], num_args: OptionNumArgs::Zero },
            &Option { names: &["--unused"], num_args: OptionNumArgs::Zero },
            &Option { names: &["--transient"], num_args: OptionNumArgs::Zero },
            &Option { names: &["--global"], num_args: OptionNumArgs::Zero },
            &Option { names: &["-n", "--dry-run"], num_args: OptionNumArgs::Zero },
        ]
    },
    &BuiltinCommand { 
        name: "_create_netns",
        accept_container: false,
        options: &[
            &Option { names: &["--dry-run"], num_args: OptionNumArgs::Zero },
            &Option { names: &["--no-iptables"], num_args: OptionNumArgs::Zero },
        ]
    },
    &BuiltinCommand { 
        name: "_destroy_netns",
        accept_container: false,
        options: &[
            &Option { names: &["--dry-run"], num_args: OptionNumArgs::Zero },
            &Option { names: &["--no-iptables"], num_args: OptionNumArgs::Zero },
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
            &Option { names: &["-f", "--file"], num_args: OptionNumArgs::Single },
        ]
    },
    &BuiltinCommand { 
        name: "_run",
        accept_container: true,
        options: &[
            &Option { names: &["-W", "--writable"], num_args: OptionNumArgs::Zero },
        ]
    },
    &BuiltinCommand { 
        name: "_run_in_netns",
        accept_container: true,
        options: &[
            &Option { names: &["--pid"], num_args: OptionNumArgs::Single },
        ]
    },
    &BuiltinCommand { 
        name: "_version_hash",
        accept_container: true,
        options: &[
            &Option { names: &["-s", "--short"], num_args: OptionNumArgs::Zero },
            &Option { names: &["-fd3"], num_args: OptionNumArgs::Zero },
        ]
    },
];

const GLOBAL_OPTIONS: &'static [&'static Option<'static>] = &[
    &Option { names: &["-E", "--env", "--environ"], num_args: OptionNumArgs::Multiple },
    &Option { names: &["-e", "--use-env"], num_args: OptionNumArgs::Multiple },
    &Option { names: &["--ignore-owner-check"], num_args: OptionNumArgs::Zero },
    &Option { names: &["--no-build"], num_args: OptionNumArgs::Zero },
    &Option { names: &["--no-version-check"], num_args: OptionNumArgs::Zero },
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

// fn main() {
//     let commands = &["paster"];
//     let containers = &["test"];
//     {
//         let args = &["-E", "1", "_clean"];
//         println!("{:?}", args);
//         let mut state = Completion::new(commands, containers);
//         for a in args {
//             state.trans(a);
//             // println!("{}", a);
//             // println!("{:?}", state);    
//         }
//         println!("{:?}", state.complete(""));
//     }
//     return;
//     {
//         let args = &["-E", "test", "_run", "-W", "test", "bash"];
//         println!("{:?}", args);
//         let mut state = Completion::new(commands, containers);
//         for a in args {
//             println!("{}", a);
//             state.trans(a);
//             println!("{:?}", state);    
//         }
//     }
//     {
//         let args = &["-E", "123", "paster", "serve"];
//         println!("{:?}", args);
//         let mut state = Completion::new(commands, containers);
//         for a in args {
//             println!("{}", a);
//             state.trans(a);
//             println!("{:?}", state);    
//         }
//     }
// }
