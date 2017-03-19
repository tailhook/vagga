use std::io::{stdout, stderr, Write};

use argparse::{ArgumentParser, StoreTrue};

use config::Config;


pub fn print_list(config: &Config, mut args: Vec<String>)
    -> Result<i32, String>
{
    let mut all = false;
    let mut builtin = false;
    let mut hidden = false;
    let mut containers = false;
    let mut zsh = false;
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
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(x) => return Ok(x),
        }
    }
    if containers {
        for (cname, _) in config.containers.iter() {
            println!("{}", cname);
        }
    } else {
        let mut commands: Vec<(String, String)> = config.commands.iter()
            .map(|(name, cmd)|
                (name.to_string(),
                 cmd.description().unwrap_or(&"".to_string()).to_string()))
            .collect();
        // TODO(tailhook) fetch builtins from completion code
        commands.push(
            ("_build".to_string(),
             "Build a container".to_string()));
        commands.push(
            ("_run".to_string(),
             "Run arbitrary command, optionally building container".to_string()));
        commands.push((
            "_clean".to_string(),
            "Clean containers and build artifacts".to_string()));
        commands.push((
            "_list".to_string(),
            "List of built-in commands".to_string()));
        commands.push((
            "_base_dir".to_string(),
            "Display a directory which contains vagga.yaml".to_string()));
        commands.push((
            "_relative_work_dir".to_string(),
            "Display a relative path from the current \
            working directory to the directory \
            containing vagga.yaml".to_string()));
        commands.push((
            "_update_symlinks".to_string(),
            "Updates symlinks to vagga for commands having ``symlink-name`` \
            in this project".to_string()));

        let mut out = stdout();
        for (name, description) in commands {
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
            }
        }
    }
    return Ok(0);
}

pub fn print_help(config: &Config)
    -> Result<i32, String>
{
    let mut err = stderr();
    writeln!(&mut err, "Available commands:").ok();
    for (k, cmd) in config.commands.iter() {
        if k.starts_with("_") {
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
        err.write_all(b"\n").ok();
    }
    Ok(127)
}
