use std::io::{stdout, stderr};

use argparse::{ArgumentParser, StoreTrue};

use config::Config;


pub fn print_list(config: &Config, args: Vec<String>)
    -> Result<i32, String>
{
    let mut all = false;
    let mut builtin = false;
    let mut hidden = false;
    {
        let mut ap = ArgumentParser::new();
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
    let mut out = stdout();
    for (k, cmd) in config.commands.iter() {
        out.write_str(k.as_slice()).ok();
        match cmd.description() {
            Some(ref val) => {
                if k.len() > 19 {
                    out.write_str("\n                    ").ok();
                } else {
                    for _ in range(k.len(), 19) {
                        out.write_char(' ').ok();
                    }
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
            "_clean              Clean containers and build artifacts\n",
            "_list               List of built-in commands\n",
        )).ok();
    }
    return Ok(0);
}
