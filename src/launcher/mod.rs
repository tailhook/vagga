use std::env;
use std::io::{stdout, stderr, Write};
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
mod sphinx;

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

pub fn run(input_args: Vec<String>) -> i32 {
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

    let mut commands = Vec::<String>::new();
    let mut cname = "".to_string();
    let mut args = vec!();
    let mut set_env = Vec::<String>::new();
    let mut propagate_env = Vec::<String>::new();
    let mut bmode = Default::default();
    let mut owner_check = true;
    let mut prerequisites = true;
    let mut isolate_network = false;

    let export_command = input_args.get(0).and_then(check_export);
    if !int_settings.run_symlinks_as_commands || export_command.is_none() {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Runs a command in container, optionally builds container if that
            does not exists or outdated.

            Run `vagga` without arguments to see the list of commands.
            ");
        ap.add_option(&["-V", "--version"],
            Print(env!("VAGGA_VERSION").to_string()),
            "Show vagga version and exit");
        ap.refer(&mut set_env)
          .add_option(&["-E", "--env", "--environ"], Collect,
                "Set environment variable for running command")
          .metavar("NAME=VALUE");
        ap.refer(&mut propagate_env)
          .add_option(&["-e", "--use-env"], Collect,
                "Propagate variable VAR into command environment")
          .metavar("VAR");
        ap.refer(&mut owner_check)
          .add_option(&["--ignore-owner-check"], StoreFalse,
                "Ignore checking owner of the project directory");
        ap.refer(&mut prerequisites)
            .add_option(&["--no-prerequisites"], StoreFalse,
            "Run only specified command(s), don't run prerequisites");
        ap.refer(&mut isolate_network)
            .add_option(
                &["--isolate-network", "--no-network", "--no-net"],
                StoreTrue,
                "Run command(s) inside isolated network");
        build_mode(&mut ap, &mut bmode);
        ap.refer(&mut commands)
          .add_option(&["-m", "--run-multi"], List, "
            Run the following list of commands. Each without an arguments.
            When any of them fails, stop the chain. Basically it's the
            shortcut to `vagga cmd1 && vagga cmd2` except containers for
            `cmd2` are built beforehand, for your convenience. Also builtin
            commands (those starting with underscore) do not work with
            `vagga -m`");
        ap.refer(&mut cname)
          .add_argument("command", Store,
                "A vagga command to run");
        ap.refer(&mut args)
          .add_argument("args", List,
                "Arguments for the command");
        ap.stop_on_first_argument(true);
        ap.silence_double_dash(false);
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
                    cname = name.clone();
                }
                _ => {}
            }
        }
        if cname == "" {
            writeln!(&mut err, "Can't find command {:?}", export_command).ok();
            return 127;
        }
        args = input_args;
        args.remove(0);  // the command itself
    }

    if &cname[..] == "_network" {
        args.insert(0, "vagga _network".to_string());
        return ::network::run(args);
    }

    if owner_check {
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

    for k in propagate_env.into_iter() {
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
    for pair in set_env.into_iter() {
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
        build_mode: bmode,
        prerequisites: prerequisites,
        isolate_network: isolate_network,
    };

    if commands.len() > 0 {
        let result = user::run_multiple_commands(&context, commands);
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

    let result:Result<i32, String> = match &cname[..] {
        "" => {
            list::print_help(&context.config)
        }
        "_create_netns" => {
            network::create_netns(&context.config, args)
        }
        "_destroy_netns" => {
            network::destroy_netns(&context.config, args)
        }
        "_list" => {
            list::print_list(&context.config, args)
        }
        "_version_hash" => {
            underscore::version_hash(&context, &cname, args)
        }
        "_build_shell" | "_clean" | "_check_overlayfs_support" => {
            underscore::passthrough(&context, &cname, args)
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
            build::build_command(&context, args)
        }
        "_run" => {
            underscore::run_command(&context, args)
        }
        "_run_in_netns" => {
            underscore::run_in_netns(&context, cname, args)
        }
        "_pack_image" => {
            pack::pack_command(&context, args)
        }
        "_push_image" => {
            push::push_command(&context, args)
        }
        "_init_storage_dir" => {
            storage::init_dir(&context.ext_settings, &cfg_dir, args)
        }
        "_compgen" => {
            completion::generate_completions(&context.config, args)
        }
        "_help" => {
            completion::generate_command_help(&context, args)
        }
        "_update_symlinks" => {
            commands::update_symlinks(&context, args)
        }
        "_capsule" => {
            ::capsule::run_command(&context, args)
        }
        _ => {
            user::run_user_command(&context, cname, args)
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
