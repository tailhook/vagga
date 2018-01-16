use std::io::{stdout, stderr, Write};
use std::rc::Rc;
use std::iter::once;
use std::collections::HashSet;

use argparse::{ArgumentParser, StoreTrue, StoreFalse};

use config::Config;


pub fn print_list(config: &Config, mut args: Vec<String>)
    -> Result<i32, String>
{
    let mut all = false;
    let mut builtin = false;
    let mut hidden = false;
    let mut aliases = true;
    let mut containers = false;
    let mut zsh = false;
    let mut verbose = false;
    {
        args.insert(0, String::from("vagga _list"));
        let mut ap = ArgumentParser::new();
        ap.refer(&mut containers)
            .add_option(&["--containers"], StoreTrue,
                "Show containers instead of commands");
        ap.refer(&mut all)
            .add_option(&["-A", "--all"], StoreTrue,
                "Show all commands");
        ap.refer(&mut builtin)
            .add_option(&["--builtin"], StoreTrue,
                "Show built-in commands (starting with underscore)");
        ap.refer(&mut hidden)
            .add_option(&["--hidden"], StoreTrue,
                "Show hidden commands");
        ap.refer(&mut zsh)
            .add_option(&["--zsh"], StoreTrue,
                "Use zsh completion compatible format");
        ap.refer(&mut aliases)
            .add_option(&["--no-aliases"], StoreFalse,
                "Do not show command aliases");
        ap.refer(&mut verbose)
            .add_option(&["-v", "--verbose"], StoreTrue,
                "Verbose output (show source files
                 for containers and commands)");
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(x) => return Ok(x),
        }
    }
    if containers {
        for (cname, container) in config.containers.iter() {
            println!("{}", cname);
            if let Some(ref src) = container.source {
                if verbose {
                    println!("{:19} (from {:?})", " ", &src);
                }
            }
        }
    } else {
        let ref builtins = Some(Rc::new("<builtins>".into()));
        let mut commands: Vec<_> = config.commands.iter()
            .map(|(name, cmd)|
                (&name[..], if aliases { cmd.aliases() } else { &[][..] },
                 cmd.description().unwrap_or(&"".to_string()).to_string(),
                 cmd.source()))
            .collect();
        // TODO(tailhook) fetch builtins from completion code
        commands.push(
            ("_build", &[][..],
             "Build a container".to_string(),
             &builtins));
        commands.push(
            ("_run", &[][..],
             "Run arbitrary command, optionally building container".to_string(),
             &builtins));
        commands.push((
            "_clean", &[][..],
            "Clean containers and build artifacts".to_string(),
            &builtins));
        commands.push((
            "_list", &[][..],
            "List of built-in commands".to_string(),
            &builtins));
        commands.push((
            "_base_dir", &[][..],
            "Display a directory which contains vagga.yaml".to_string(),
            &builtins));
        commands.push((
            "_relative_work_dir", &[][..],
            "Display a relative path from the current \
            working directory to the directory \
            containing vagga.yaml".to_string(),
            &builtins));
        commands.push((
            "_update_symlinks", &[][..],
            "Updates symlinks to vagga for commands having ``symlink-name`` \
            in this project".to_string(),
            &builtins));

        let mut out = stdout();
        for (orig_name, aliases, description, source) in commands {
            if (orig_name.starts_with("_")) && !(hidden || all) {
                continue;
            }
            let all_aliases = once(orig_name).chain(
                aliases.iter().map(|x| &x[..])
                .filter(|&x| !config.commands.contains_key(x)));
            for name in all_aliases {
                if name.starts_with("_") && !(hidden || all) {
                    continue;
                }
                if zsh {
                    let descr_line = description
                        .lines().next().unwrap_or(&"");
                    out.write_all(name.as_bytes()).ok();
                    out.write_all(":".as_bytes()).ok();
                    out.write_all(descr_line.as_bytes()).ok();
                    out.write_all(b"\n").ok();
                } else {
                    out.write_all(name.as_bytes()).ok();
                    if name.len() > 19 {
                        out.write_all(b"\n                    ").ok();
                    } else {
                        for _ in name.len()..19 {
                            out.write_all(b" ").ok();
                        }
                        out.write_all(b" ").ok();
                    }
                    if description.contains("\n") {
                        for line in description.lines() {
                            out.write_all(line.as_bytes()).ok();
                            out.write_all(b"\n                    ").ok();
                        };
                    } else {
                        out.write_all(description.as_bytes()).ok();
                    }
                    out.write_all(b"\n").ok();
                    if let Some(ref src) = *source {
                        if verbose {
                            if name != orig_name {
                                println!("{:19} (from {:?}:{})", " ",
                                    &src, orig_name);
                            } else {
                                println!("{:19} (from {:?})", " ", &src);
                            }
                        }
                    }
                }
            }
        }
    }
    return Ok(0);
}

pub fn print_help(config: &Config)
    -> Result<i32, String>
{
    let mut err = stderr();
    let mut visited_groups = HashSet::new();
    for (_, group) in &config.commands {
        if visited_groups.contains(&group.group_title()) {
            continue;
        }
        visited_groups.insert(group.group_title());
        if visited_groups.len() > 1 {
            err.write_all(b"\n").ok();
        }
        writeln!(&mut err, "{}:",
            group.group_title().unwrap_or("Available commands")).ok();
        for (k, cmd) in config.commands.iter() {
            if k.starts_with("_") || cmd.group_title() != group.group_title() {
                continue;
            }
            write!(&mut err, "    {}", k).ok();
            match cmd.description() {
                Some(ref val) => {
                    if k.len() > 19 {
                        err.write_all(b"\n                        ").ok();
                    } else {
                        for _ in k.len()..19 {
                            err.write_all(b" ").ok();
                        }
                        err.write_all(b" ").ok();
                    }
                    if val.contains("\n") {
                        for line in val.lines() {
                            err.write_all(line.as_bytes()).ok();
                            err.write_all(b"\n                        ").ok();
                        };
                    } else {
                        err.write_all(val.as_bytes()).ok();
                    }
                }
                None => {}
            }
            let aliases = cmd.aliases()
                .iter().filter(|&x| !config.commands.contains_key(x))
                .map(|x| &x[..])
                .collect::<Vec<_>>();
            if aliases.len() > 0 {
                err.write_all(b"\n                        ").ok();
                write!(&mut err, "(aliases: {})",
                    aliases.join(", ")).ok();
            }
            err.write_all(b"\n").ok();
        }
    }
    Ok(127)
}
