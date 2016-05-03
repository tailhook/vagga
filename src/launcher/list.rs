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
        let mut out = stdout();
        for (k, cmd) in config.commands.iter() {
            out.write_all(k.as_bytes()).ok();
            match cmd.description() {
                Some(ref val) => {
                    if k.len() > 19 {
                        out.write_all(b"\n                    ").ok();
                    } else {
                        for _ in k.len()..19 {
                            out.write_all(b" ").ok();
                        }
                        out.write_all(b" ").ok();
                    }
                    out.write_all(val.as_bytes()).ok();
                }
                None => {}
            }
            out.write_all(b"\n").ok();
        }

        if all || builtin {
            // TODO(tailhook) fetch builtins from completion code
            write!(&mut out, "\
                _build              Build a container\n\
                _run                Run arbitrary command, \
                                    optionally building container\n\
                _clean              Clean containers and build artifacts\n\
                _list               List of built-in commands\n\
            ").ok();
        }
    }
    return Ok(0);
}
