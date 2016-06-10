use config::volumes::Volume::{self, Container};
use launcher::build::build_container;
use launcher::Context;


pub fn prepare_volumes<'x, I>(volumes: I, context: &Context)
    -> Result<(), String>
    where I: Iterator<Item=&'x Volume>
{
    for v in volumes {
        match *v {
            Container(ref name) => {
                try!(build_container(context, name, context.build_mode));
            }
            _ => {}
        }
    }
    Ok(())
}
