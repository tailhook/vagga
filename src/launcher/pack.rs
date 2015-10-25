use unshare::Command;

use options::pack::{Options};
use config::settings::Settings;
use launcher::build::build_container;
use launcher::wrap::Wrapper;


pub fn pack_command(settings: &Settings, args: Vec<String>)
    -> Result<i32, String>
{
    let mut cmdline = args.clone();
    cmdline.insert(0, "vagga _pack_image".to_string());
    let opt = match Options::parse(&cmdline) {
        Ok(x) => x,
        Err(code) => return Ok(code),
    };

    let ver = try!(build_container(settings, &opt.name, opt.build_mode));

    let mut cmd: Command = Wrapper::new(Some(&ver), &settings);
    cmd.userns();
    cmd.arg("_pack_image").args(&args);
    cmd.run()
}
