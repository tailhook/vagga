use std::io::{stdout, stderr};

use argparse::{ArgumentParser, Store, StoreTrue};

use capsule::Context;
use launcher::build::build_container;


pub fn build_command(context: &Context, args: Vec<String>)
    -> Result<i32, String>
{
    let mut name: String = "".to_string();
    let mut force: bool = false;
    {
        let mut cmdline = args.clone();
        cmdline.insert(0, String::from("vagga _capsule build"));
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
    build_container(context, &name, context.build_mode, true)
    .map(|v| debug!("Container {:?} is built with version {:?}", name, v))
    .map(|()| 0)
}
