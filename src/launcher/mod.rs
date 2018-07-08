use std::env;
use std::io::{stdout, stderr, Write};
use std::path::{Path, PathBuf};
use std::fs::metadata;
use std::os::unix::fs::MetadataExt;

use libc::getuid;
use argparse::{ArgumentParser, Store, List, Collect, Print, StoreFalse, StoreTrue};

use options::build_mode::{build_mode, BuildMode};
use config::{Config, ConfigError, Settings, find_config};
use config::read_settings::{read_settings, MergedSettings};
use launcher::environ::set_initial_vaggaenv_vars;

mod list;
mod integration;
#[cfg(feature="containers")] mod user;
#[cfg(feature="containers")] mod pack;
#[cfg(feature="containers")] mod push;
#[cfg(feature="containers")] pub mod build;
#[cfg(feature="containers")] mod storage;
#[cfg(feature="containers")] mod underscore;
#[cfg(feature="containers")] mod completion;
#[cfg(feature="containers")] mod commands;
#[cfg(feature="containers")] mod storage_dir;

#[cfg(feature="containers")] mod supervisor;
#[cfg(feature="containers")] mod simple;
#[cfg(feature="containers")] mod capsule;

#[cfg(feature="containers")] pub mod wrap;
#[cfg(feature="containers")] pub mod system;
#[cfg(feature="containers")] pub mod network;
mod environ;
mod options;
#[cfg(feature="containers")] mod prerequisites;
#[cfg(feature="containers")] mod socket;
#[cfg(feature="containers")] mod volumes;


pub struct Context {
    pub config: Config,
    pub settings: Settings,
    pub ext_settings: MergedSettings,
    pub workdir: PathBuf,
    pub config_dir: PathBuf,
    pub build_mode: BuildMode,
    pub prerequisites: bool,
    pub isolate_network: bool,
    pub containers_only: bool,
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

    let mut commands = Vec::<String>::new();
    let mut cname = "".to_string();
    let mut args = vec!();
    let mut set_env = Vec::<String>::new();
    let mut propagate_env = Vec::<String>::new();
    let mut bmode = Default::default();
    let mut owner_check = true;
    let mut prerequisites = true;
    let mut containers_only = false;
    let mut isolate_network = false;

    let export_command = input_args.get(0).and_then(check_export);
    let (cli_ok, cli_out, cli_err) = {
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
        ap.refer(&mut containers_only)
            .add_option(&["--containers-only"], StoreTrue,
            "Only build container images needed to run commands,
             don't run any commands.");
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
        let mut cli_out = Vec::new();
        let mut cli_err = Vec::new();
        match ap.parse(input_args.clone(), &mut cli_out, &mut cli_err) {
            Err(0) if export_command.is_none() => {
                stdout().write_all(&cli_out).ok();
                stderr().write_all(&cli_err).ok();
                return 0;
            }
            r => (r, cli_out, cli_err),
        }
    };

    let workdir = env::current_dir().unwrap();

    let (config, cfg_dir) = match find_config(&workdir, true) {
        Ok(tup) => tup,
        Err(e@ConfigError::NotFound(_)) => {
            eprintln!("{}", e);
            eprintln!("Hint: you need to `cd` into a project, \
                       or create a file `vagga.yaml` in it.");
            return 126;
        }
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

    let no_symlink =
        !cfg!(feature="containers")
        || !int_settings.run_symlinks_as_commands
        || export_command.is_none();

    if no_symlink {
        stdout().write_all(&cli_out).ok();
        stderr().write_all(&cli_err).ok();
        match cli_ok {
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

        #[cfg(feature="containers")]
        return ::network::run(args);

        #[cfg(not(feature="containers"))]
        {
            eprintln!("vagga was built without containers support");
            return 127;
        }
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

    match set_initial_vaggaenv_vars(propagate_env, set_env, &ext_settings) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Error propagating environment: {}", e);
            return 126;
        }
    }

    let context = Context {
        config,
        settings: int_settings,
        ext_settings,
        workdir: int_workdir.to_path_buf(),
        config_dir: cfg_dir.to_path_buf(),
        build_mode: bmode,
        prerequisites,
        isolate_network,
        containers_only,
    };

    if commands.len() > 0 {
        #[cfg(feature="containers")]
        {
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
        #[cfg(not(feature="containers"))]
        {
            eprintln!("vagga was built without containers support");
            return 127;
        }
    }

    let result:Result<i32, String> = match &cname[..] {
        "" => {
            list::print_help(&context.config)
        }
        #[cfg(feature="containers")]
        "_create_netns" => {
            network::create_netns(&context.config, args)
        }
        #[cfg(feature="containers")]
        "_destroy_netns" => {
            network::destroy_netns(&context.config, args)
        }
        "_list" => {
            list::print_list(&context.config, args)
        }
        "_dump_config" => {
            integration::dump_config(&context.config, args)
        }
        #[cfg(feature="containers")]
        "_version_hash" => {
            underscore::version_hash(&context, &cname, args)
        }
        #[cfg(feature="containers")]
        "_build_shell" | "_clean" | "_check_overlayfs_support" |
        "_hardlink" | "_verify" => {
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
        #[cfg(feature="containers")]
        "_build" => {
            build::build_command(&context, args)
        }
        #[cfg(feature="containers")]
        "_run" => {
            underscore::run_command(&context, args)
        }
        #[cfg(feature="containers")]
        "_run_in_netns" => {
            underscore::run_in_netns(&context, cname, args)
        }
        #[cfg(feature="containers")]
        "_pack_image" => {
            pack::pack_command(&context, args)
        }
        #[cfg(feature="containers")]
        "_push_image" => {
            push::push_command(&context, args)
        }
        #[cfg(feature="containers")]
        "_init_storage_dir" => {
            storage::init_dir(&context.ext_settings, &cfg_dir, args)
        }
        // TODO(tailhook) allow completion in no-containers mode
        #[cfg(feature="containers")]
        "_compgen" => {
            completion::generate_completions(&context.config, args)
        }
        // TODO(tailhook) allow help in no-containers mode
        #[cfg(feature="containers")]
        "_help" => {
            completion::generate_command_help(&context, args)
        }
        #[cfg(feature="containers")]
        "_update_symlinks" => {
            commands::update_symlinks(&context, args)
        }
        #[cfg(feature="containers")]
        "_capsule" => {
            ::capsule::run_command(&context, args)
        }
        #[cfg(feature="containers")]
        _ => {
            user::run_user_command(&context, cname, args)
        }
        #[cfg(not(feature="containers"))]
        _ => {
            eprintln!("vagga was built without containers support");
            return 127;
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
