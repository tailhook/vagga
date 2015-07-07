use std::io::{stdout, stderr};

use argparse::{ArgumentParser, Store, StoreTrue};

use config::Config;

use super::user;


pub fn build_container(config: &Config, name: &String) -> Result<(), String> {
    build_command(config, vec!(name.clone())).map(|_| ())
}

pub fn build_command(config: &Config, mut args: Vec<String>)
    -> Result<i32, String>
{
    let mut name: String = "".to_string();
    let mut force: bool = false;
    {
        let mut cmdline = args.clone();
        cmdline.insert(0, "vagga _build".to_string());
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Internal vagga tool to setup basic system sandbox
            ");
        ap.refer(&mut name)
            .add_argument("container_name", Store,
                "Container name to build");
        ap.refer(&mut force)
            .add_option(&["--force"], StoreTrue,
                "Force build even if container is considered up to date");
        match ap.parse(cmdline, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => {
                return Ok(122);
            }
        }
    }
    match user::run_wrapper(None, "_build".to_string(), args, true) {
        Ok(0) => Ok(0),
        Ok(x) => Err(format!("Build returned {}", x)),
        Err(e) => Err(e),
    }
}
