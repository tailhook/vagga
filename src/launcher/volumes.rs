use config::volumes::SnapshotInfo;
use config::volumes::Volume::{self, Container, Snapshot};
use launcher::build::build_container;
use launcher::Context;


pub fn prepare_volumes<'x, I>(volumes: I, context: &Context)
    -> Result<(), String>
    where I: Iterator<Item=&'x Volume>
{
    for v in volumes {
        match *v {
            Container(ref name) |
            Snapshot(SnapshotInfo { container: Some(ref name), .. }) => {
                build_container(context, name, context.build_mode)?;
            }
            _ => {}
        }
    }
    Ok(())
}
