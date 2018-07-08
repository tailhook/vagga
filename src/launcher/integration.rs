use std::io::{stdout, stderr};

use argparse::{ArgumentParser, StoreTrue};
use serde_json;

use config::Config;


pub fn dump_config(config: &Config, mut args: Vec<String>)
    -> Result<i32, String>
{
    let mut pretty = false;
    {
        args.insert(0, String::from("vagga _dump_config"));
        let mut ap = ArgumentParser::new();
        ap.refer(&mut pretty)
            .add_option(&["--pretty"], StoreTrue,
                "Prettify printed json");
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(x) => return Ok(x),
        }
    }
    if pretty {
        println!("{}", serde_json::to_string_pretty(config)
            .expect("can serialize config"));
    } else {
        println!("{}", serde_json::to_string(config)
            .expect("can serialize config"));
    }
    return Ok(0);
}
