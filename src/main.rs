use argparse::{ArgumentParser, StoreOption, StoreTrue, List};
use std::os::{getcwd, args};
use std::io::stdio::stderr;
use std::io::stdio::stdout;

use super::config::find_config;
use super::config::{Shell, Plain, Supervise};
use super::build::build_command;
use super::run::run_command_line;
use super::chroot::run_chroot;
use super::userns::run_userns;
use super::clean::{run_do_rm, run_clean};
use super::commands::shell::run_shell_command;
use super::commands::command::run_plain_command;
use super::commands::supervise::run_supervise_command;
use super::utils::json::extract_json;
use super::env::Environ;
use super::options::env_options;
use super::settings::{read_settings, set_variant};


pub fn print_list(env: &mut Environ, args: Vec<String>)
    -> Result<int, String>
{
    let mut all = false;
    let mut builtin = false;
    let mut hidden = false;
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut all)
            .add_option(["-A", "--all"], box StoreTrue,
                "Show all commands");
        ap.refer(&mut builtin)
            .add_option(["--builtin"], box StoreTrue,
                "Show built-in commands (starting with underscore)");
        ap.refer(&mut hidden)
            .add_option(["--hidden"], box StoreTrue,
                "Show hidden commands");
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(x) => return Ok(x),
        }
    }
    let mut out = stdout();
    for (k, cmd) in env.config.commands.iter() {
        out.write_str(k.as_slice()).ok();
        match cmd.description {
            Some(ref val) => {
                for _ in range(k.len(), 20) {
                    out.write_char(' ').ok();
                }
                out.write_str(val.as_slice()).ok();
            }
            None => {}
        }
        out.write_char('\n').ok();
    }

    if all || builtin {
        out.write_str(concat!(
            "_build              Build a container\n",
            "_run                Run arbitrary command, ",
                                "optionally building container\n",
            "_setvariant         Override default variant for ",
                                "subsequent commands\n",
            "_clean              Clean containers and build artifacts\n",
            "_list               List of built-in commands\n",
        )).ok();

        if all || hidden {
            out.write_str(concat!(
                "_chroot             Do change root into arbitrary folder\n",
                "_userns             Setup user namespace\n",
                "_extract_json       Extract some values from JSON stream\n",
                "__rm                Remove dir under user namespace\n",
            )).ok();
        }
    }
    return Ok(0);
}


pub fn run() -> int {
    let mut err = stderr();
    let workdir = getcwd();

    let (config, project_root) = match find_config(&workdir) {
        Ok(tup) => tup,
        Err(e) => {
            err.write_line(e.as_slice()).ok();
            return 126;
        }
    };
    let mut env = match Environ::new(project_root, config) {
        Ok(env) => env,
        Err(e) => {
            err.write_line(e.as_slice()).ok();
            return 121;
        }
    };
    read_settings(&mut env);

    let mut cmd: Option<String> = None;
    let mut args: Vec<String> = Vec::new();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Runs a command in container, optionally builds container if that
            does not exists or outdated.

            Run `vagga` without arguments to see the list of commands.
            ");
        ap.refer(&mut cmd)
          .add_argument("command", box StoreOption::<String>,
                "A vagga command to run");
        ap.refer(&mut args)
          .add_argument("args", box List::<String>,
                "Arguments for the command");
        env_options(&mut env, &mut ap);
        ap.stop_on_first_argument(true);
        match ap.parse_args() {
            Ok(()) => {}
            Err(0) => return 0,
            Err(_) => return 122,
        }
    }

    if cmd.is_none() {
        err.write_line("Available commands:").ok();
        for (k, cmd) in env.config.commands.iter() {
            err.write_str("    ").ok();
            err.write_str(k.as_slice()).ok();
            match cmd.description {
                Some(ref val) => {
                    for _ in range(k.len(), 20) {
                        err.write_char(' ').ok();
                    }
                    err.write_str(val.as_slice()).ok();
                }
                None => {}
            }
            err.write_char('\n').ok();
        }
        return 127;
    }

    let cname = cmd.unwrap();
    args.insert(0, "vagga ".to_string() + cname);
    let result = match cname.as_slice() {
        // Commands for users
        "_build" => build_command(&mut env, args),
        "_run" => run_command_line(&mut env, args),
        "_setv" | "_setvariant" => set_variant(&mut env, args),
        "_clean" => run_clean(&mut env, args),
        "_list" => print_list(&mut env, args),

        // Commands for builders
        "_chroot" => run_chroot(&mut env, args),
        "_userns" => run_userns(&mut env, args),
        "_extract_json" => extract_json(&mut env, args),

        // Commands run by vagga in namespaces
        "__rm" => run_do_rm(&mut env, args),

        _ => {
            let fun = match env.config.commands.find(&cname) {
                Some(ref cmd) => {
                    match cmd.execute {
                        Shell(_) => run_shell_command,
                        Plain(_) => run_plain_command,
                        Supervise(_, _) => run_supervise_command,
                    }
                }
                None => {
                    err.write_line(
                        format!("Unknown command {}", cname).as_slice()).ok();
                    return 127;
                }
            };
            fun(&mut env, &cname, args)
        }
    };
    match result {
        Ok(rc) => {
            return rc;
        }
        Err(text) =>  {
            err.write_line(text.as_slice()).ok();
            return 121;
        }
    }
}
