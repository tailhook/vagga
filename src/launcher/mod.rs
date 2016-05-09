use std::env;
use std::io::{stderr, Write};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::fs::metadata;
use std::os::unix::fs::MetadataExt;

use libc::getuid;
use unshare::Command;

use options::build_mode::{build_mode, BuildMode};
use config::{Config, Settings, find_config};
use config::read_settings::read_settings;
use process_util::convert_status;
use argparse::{ArgumentParser, Store, List, Collect, Print, StoreFalse};
use super::path_util::ToRelative;
use self::wrap::Wrapper;

mod list;
mod user;
mod pack;
mod push;
mod build;
mod storage;
mod underscore;
mod completion;

mod supervisor;
mod simple;

mod wrap;
mod network;
mod volumes;
mod prerequisites;


pub struct Context {
    config: Config,
    settings: Settings,
    workdir: PathBuf,
    build_mode: BuildMode,
    prerequisites: bool,
}

pub fn run() -> i32 {
    let mut err = stderr();
    let mut commands = Vec::<String>::new();
    let mut cname = "".to_string();
    let mut args = vec!();
    let mut set_env = Vec::<String>::new();
    let mut propagate_env = Vec::<String>::new();
    let mut bmode = Default::default();
    let mut owner_check = true;
    let mut prerequisites = true;
    {
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
        match ap.parse_args() {
            Ok(()) => {}
            Err(0) => return 0,
            Err(_) => return 122,
        }
    }

    if &cname[..] == "_network" {
        args.insert(0, "vagga _network".to_string());
        return ::network::run(args);
    }

    let workdir = env::current_dir().unwrap();

    let (config, cfg_dir) = match find_config(&workdir) {
        Ok(tup) => tup,
        Err(e) => {
            writeln!(&mut err, "{}", e).ok();
            return 126;
        }
    };

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
    let int_workdir = workdir.rel_to(&cfg_dir)
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

    if commands.len() > 0 {
        let result = user::run_multiple_commands(&Context {
            config: config,
            settings: int_settings,
            workdir: int_workdir.to_path_buf(),
            build_mode: bmode,
            prerequisites: prerequisites,
        }, commands);
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
            writeln!(&mut err, "Available commands:").ok();
            for (k, cmd) in config.commands.iter() {
                write!(&mut err, "    {}", k).ok();
                match cmd.description() {
                    Some(ref val) => {
                        if k.len() > 19 {
                            write!(&mut err, "\n                        ").ok();
                        } else {
                            for _ in k.len()..19 {
                                err.write_all(b" ").ok();
                            }
                            err.write_all(b" ").ok();
                        }
                        err.write_all(val[..].as_bytes()).ok();
                    }
                    None => {}
                }
                err.write_all(b"\n").ok();
            }
            return 127;
        }
        "_create_netns" => {
            network::create_netns(&config, args)
        }
        "_destroy_netns" => {
            network::destroy_netns(&config, args)
        }
        "_list" => {
            list::print_list(&config, args)
        }
        "_build_shell" | "_clean" | "_version_hash" |
        "_check_overlayfs_support" => {
            let mut cmd: Command = Wrapper::new(None, &int_settings);
            cmd.workdir(&int_workdir);
            cmd.userns();
            cmd.arg(&cname).args(&args);
            cmd.status()
            .map(convert_status)
            .map_err(|e| format!("Error running `vagga_wrapper {}`: {}",
                                 cname, e))
        }
        "_build" => {
            build::build_command(&int_settings, args)
        }
        "_run" => {
            underscore::run_command(&int_settings, &int_workdir, args, bmode)
        }
        "_run_in_netns" => {
            underscore::run_in_netns(&int_settings, &int_workdir, cname, args,
                bmode)
        }
        "_pack_image" => {
            pack::pack_command(&int_settings, args)
        }
        "_push_image" => {
            push::push_command(&ext_settings, &int_settings, args)
        }
        "_init_storage_dir" => {
            storage::init_dir(&ext_settings, &cfg_dir, args)
        }
        "_compgen" => {
            completion::generate_completions(&config, args)
        }
        _ => {
            user::run_user_command(&Context {
                config: config,
                settings: int_settings,
                workdir: int_workdir.to_path_buf(),
                build_mode: bmode,
                prerequisites: prerequisites,
            }, cname, args)
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

pub fn main() {
    let val = run();
    exit(val);
}
