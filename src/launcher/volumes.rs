use config::Settings;
use config::containers::Volume::{self, Container};
use options::build_mode::BuildMode;
use launcher::build::build_container;


pub fn prepare_volumes<'x, I>(volumes: I, settings: &Settings,
    build_mode: BuildMode)
    -> Result<(), String>
    where I: Iterator<Item=&'x Volume>
{
    for v in volumes {
        match *v {
            Container(ref name) => {
                try!(build_container(settings, name, build_mode));
            }
            _ => {}
        }
    }
    Ok(())
}
