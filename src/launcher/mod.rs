use std::env;
use std::ffi::OsStr;
use std::io::{self, stdout, stderr, Write};
use std::path::{Path, PathBuf};
use std::fs::metadata;
use std::os::unix::fs::MetadataExt;

use libc::getuid;

use options::build_mode::{build_mode, BuildMode};
use config::{Config, Settings, find_config};
use config::read_settings::{read_settings, MergedSettings};
use argparse::{ArgumentParser, Store, List, Collect, Print, StoreFalse, StoreTrue};

mod list;
mod user;
mod pack;
mod push;
pub mod build;
mod storage;
mod underscore;
mod completion;
mod commands;
mod storage_dir;

mod supervisor;
mod simple;
mod capsule;

pub mod wrap;
pub mod system;
pub mod network;
mod socket;
mod volumes;
mod options;
mod prerequisites;


pub struct Context {
    pub config: Config,
    pub settings: Settings,
    pub ext_settings: MergedSettings,
    pub workdir: PathBuf,
    pub config_dir: PathBuf,
    pub build_mode: BuildMode,
    pub prerequisites: bool,
    pub isolate_network: bool
}

fn check_export(cmd: &String) -> Option<String> {
    if cmd == "/proc/self/exe" || cmd == "vagga" {
        return None;
    }
    match cmd.rfind("/") {
        Some(slash) => {
            let exe = &cmd[slash+1..];
            if exe == "vagga" || exe.starts_with("vagga_") {
                None
            } else {
                Some(exe.to_owned())
            }
        }
        None => Some(cmd.clone()),
    }
}

struct LauncherOptions {
    commands: Vec<String>,
    cname: String,
    args: Vec<String>,
    set_env: Vec<String>,
    propagate_env: Vec<String>,
    bmode: BuildMode,
    owner_check: bool,
    prerequisites: bool,
    isolate_network: bool,
}

impl Default for LauncherOptions {
    fn default() -> LauncherOptions {
        LauncherOptions {
            commands: vec!(),
            cname: "".to_string(),
            args: vec!(),
            set_env: vec!(),
            propagate_env: vec!(),
            bmode: Default::default(),
            owner_check: true,
            prerequisites: true,
            isolate_network: false,
        }
    }
}

fn arg_parser<'ap>(opts: &'ap mut LauncherOptions) -> ArgumentParser<'ap> {
    let mut ap = ArgumentParser::new();
    ap.set_description("
            Runs a command in container, optionally builds container if that
            does not exists or outdated.

            Run `vagga` without arguments to see the list of commands.
            ");
    ap.add_option(&["-V", "--version"],
                  Print(env!("VAGGA_VERSION").to_string()),
                  "Show vagga version and exit");
    ap.refer(&mut opts.set_env)
        .add_option(&["-E", "--env", "--environ"], Collect,
                    "Set environment variable for running command")
        .metavar("NAME=VALUE");
    ap.refer(&mut opts.propagate_env)
        .add_option(&["-e", "--use-env"], Collect,
                    "Propagate variable VAR into command environment")
        .metavar("VAR");
    ap.refer(&mut opts.owner_check)
        .add_option(&["--ignore-owner-check"], StoreFalse,
                    "Ignore checking owner of the project directory");
    ap.refer(&mut opts.prerequisites)
        .add_option(&["--no-prerequisites"], StoreFalse,
                    "Run only specified command(s), don't run prerequisites");
    ap.refer(&mut opts.isolate_network)
        .add_option(
            &["--isolate-network", "--no-network", "--no-net"],
            StoreTrue,
            "Run command(s) inside isolated network");
    build_mode(&mut ap, &mut opts.bmode);
    ap.refer(&mut opts.commands)
        .add_option(&["-m", "--run-multi"], List, "
            Run the following list of commands. Each without an arguments.
            When any of them fails, stop the chain. Basically it's the
            shortcut to `vagga cmd1 && vagga cmd2` except containers for
            `cmd2` are built beforehand, for your convenience. Also builtin
            commands (those starting with underscore) do not work with
            `vagga -m`");
    ap.refer(&mut opts.cname)
        .add_argument("command", Store,
                      "A vagga command to run");
    ap.refer(&mut opts.args)
        .add_argument("args", List,
                      "Arguments for the command");
    ap.stop_on_first_argument(true);
    ap.silence_double_dash(false);
    ap
}

struct DevNull;

impl Write for DevNull {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn process_help_and_version(input_args: Vec<String>) -> bool {
    let arg0 = input_args.get(0).map(|n| n.as_str()).unwrap_or("").to_string();
    let _arg0_path = PathBuf::from(&arg0);
    let exe_name = _arg0_path.file_name().unwrap_or(OsStr::new(""));

    if exe_name == "vagga" {
        let mut show_help = false;
        let mut show_version = false;
        let parse_res = {
            let mut ap = ArgumentParser::new();
            ap.set_description("Show vagga version");
            ap.refer(&mut show_help)
                .add_option(&["-h", "--help"], StoreTrue,
                            "Show vagga help and exit");
            ap.refer(&mut show_version)
                .add_option(&["-V", "--version"], StoreTrue,
                            "Show vagga version and exit");
            ap.stop_on_first_argument(true);
            ap.silence_double_dash(false);
            ap.parse(input_args, &mut DevNull {}, &mut DevNull {})
        };
        match parse_res {
            Ok(()) => {
                if show_help {
                    let mut launcher_opts = LauncherOptions::default();
                    let launcher_ap = arg_parser(&mut launcher_opts);
                    launcher_ap.print_help(&arg0, &mut stdout()).unwrap();
                    return true;
                }
                if show_version {
                    println!("{}", env!("VAGGA_VERSION"));
                    return true;
                }
            },
            _ => {},
        }
    }
    false
}

pub fn run(input_args: Vec<String>) -> i32 {
    if process_help_and_version(input_args.clone()) {
        return 0;
    }

    let mut err = stderr();
    let workdir = env::current_dir().unwrap();

    let (config, cfg_dir) = match find_config(&workdir, true) {
        Ok(tup) => tup,
        Err(e) => {
            writeln!(&mut err, "{}", e).ok();
            return 126;
        }
    };
    // Not sure this is a best place, but the variable is needed for correct
    // reading of settings
    if let Some(x) = env::var_os("HOME") {
        env::set_var("_VAGGA_HOME", x);
    }

    let (ext_settings, int_settings) = match read_settings(&cfg_dir)
    {
        Ok(tup) => tup,
        Err(e) => {
            writeln!(&mut err, "{}", e).ok();
            return 126;
        }
    };

    let mut opts = LauncherOptions::default();
    let export_command = input_args.get(0).and_then(check_export);
    if !int_settings.run_symlinks_as_commands || export_command.is_none() {
        let ap = arg_parser(&mut opts);
        match ap.parse(input_args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return 0,
            Err(_) => return 122,
        }
    } else {
        let export_command = export_command.unwrap();
        for (name, cmd) in &config.commands {
            match cmd.link() {
                Some(ref lnk) if lnk.name == export_command => {
                    opts.cname = name.clone();
                }
                _ => {}
            }
        }
        if opts.cname == "" {
            writeln!(&mut err, "Can't find command {:?}", export_command).ok();
            return 127;
        }
        opts.args = input_args;
        opts.args.remove(0);  // the command itself
    }

    if &opts.cname[..] == "_network" {
        opts.args.insert(0, "vagga _network".to_string());
        return ::network::run(opts.args);
    }

    if opts.owner_check {
        let uid = unsafe { getuid() };
        match metadata(&cfg_dir) {
            Ok(ref stat) if stat.uid() == uid  => {}
            Ok(_) => {
                if uid == 0 {
                    writeln!(&mut err, "You should not run vagga as root \
                        (see http://bit.ly/err_root)").ok();
                    return 122;
                } else {
                    warn!("You are running vagga as a user \
                        different from the owner of project directory. \
                        You may not have needed permissions \
                        (see http://bit.ly/err_root)");
                }
            }
            Err(e) => {
                writeln!(&mut err, "Can't stat {:?}: {}", cfg_dir, e).ok();
                return 126;
            }
        }
    }

    let int_workdir = workdir.strip_prefix(&cfg_dir)
                             .unwrap_or(&Path::new("."));

    for k in opts.propagate_env.into_iter() {
        if k.chars().find(|&c| c == '=').is_some() {
            writeln!(&mut err, "Environment variable name \
                (for option `-e`/`--use-env`) \
                can't contain equals `=` character. \
                To set key-value pair use `-E`/`--environ` option").ok();
            return 126;
        } else {
            env::set_var(&("VAGGAENV_".to_string() + &k[..]),
                env::var_os(&k).unwrap_or(From::from("")));
        }
    }
    for pair in opts.set_env.into_iter() {
        let mut pairiter = pair[..].splitn(2, '=');
        let key = "VAGGAENV_".to_string() + pairiter.next().unwrap();
        if let Some(value) = pairiter.next() {
            env::set_var(&key, value.to_string());
        } else {
            env::remove_var(&key);
        }
    }

    let context = Context {
        config: config,
        settings: int_settings,
        ext_settings: ext_settings,
        workdir: int_workdir.to_path_buf(),
        config_dir: cfg_dir.to_path_buf(),
        build_mode: opts.bmode,
        prerequisites: opts.prerequisites,
        isolate_network: opts.isolate_network,
    };

    if opts.commands.len() > 0 {
        let result = user::run_multiple_commands(&context, opts.commands);
        match result {
            Ok(rc) => {
                return rc;
            }
            Err(text) =>  {
                writeln!(&mut err, "{}", text).ok();
                return 121;
            }
        }
    }

    let result:Result<i32, String> = match &opts.cname[..] {
        "" => {
            list::print_help(&context.config)
        }
        "_create_netns" => {
            network::create_netns(&context.config, opts.args)
        }
        "_destroy_netns" => {
            network::destroy_netns(&context.config, opts.args)
        }
        "_list" => {
            list::print_list(&context.config, opts.args)
        }
        "_version_hash" => {
            underscore::version_hash(&context, &opts.cname, opts.args)
        }
        "_build_shell" | "_clean" | "_check_overlayfs_support" => {
            underscore::passthrough(&context, &opts.cname, opts.args)
        }
        "_base_dir" => {
            println!("{}", cfg_dir.display());
            Ok(0)
        }
        "_relative_work_dir" => {
            println!("{}", int_workdir.display());
            Ok(0)
        }
        "_build" => {
            build::build_command(&context, opts.args)
        }
        "_run" => {
            underscore::run_command(&context, opts.args)
        }
        "_run_in_netns" => {
            underscore::run_in_netns(&context, opts.cname, opts.args)
        }
        "_pack_image" => {
            pack::pack_command(&context, opts.args)
        }
        "_push_image" => {
            push::push_command(&context, opts.args)
        }
        "_init_storage_dir" => {
            storage::init_dir(&context.ext_settings, &cfg_dir, opts.args)
        }
        "_compgen" => {
            completion::generate_completions(&context.config, opts.args)
        }
        "_help" => {
            completion::generate_command_help(&context, opts.args)
        }
        "_update_symlinks" => {
            commands::update_symlinks(&context, opts.args)
        }
        "_capsule" => {
            ::capsule::run_command(&context, opts.args)
        }
        _ => {
            user::run_user_command(&context, opts.cname, opts.args)
        }
    };

    match result {
        Ok(rc) => {
            return rc;
        }
        Err(text) =>  {
            writeln!(&mut err, "{}", text).ok();
            return 121;
        }
    }
}
