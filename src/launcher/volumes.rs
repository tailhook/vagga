use crate::config::volumes::SnapshotInfo;
use crate::config::volumes::Volume::{self, Container, Snapshot};
use crate::launcher::build::build_container;
use crate::launcher::Context;


pub fn prepare_volumes<'x, I>(volumes: I, context: &Context)
    -> Result<(), String>
    where I: Iterator<Item=&'x Volume>
{
    for v in volumes {
        match *v {
            Container(ref name) |
            Snapshot(SnapshotInfo { container: Some(ref name), .. }) => {
                build_container(context, name, context.build_mode, false)?;
            }
            _ => {}
        }
    }
    Ok(())
}
