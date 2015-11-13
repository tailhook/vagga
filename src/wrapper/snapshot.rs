use std::path::Path;

use config::containers::SnapshotInfo;
use file_util::create_dir;
use container::mount::{bind_mount, mount_tmpfs, unmount};
use container::util::{copy_dir};


pub fn make_snapshot(dest: &Path, info: &SnapshotInfo)
    -> Result<(), String>
{
    let tmp = Path::new("/tmp/mnt");
    try_msg!(create_dir(&tmp, true),
        "Error creating temporary mountpoint: {err}");
    try!(bind_mount(&dest, &tmp));
    try!(mount_tmpfs(&dest,
        &format!("size={},mode=0755", info.size)));
    try_msg!(copy_dir(&tmp, dest),
        "Error copying directory: {err}");
    try!(unmount(tmp));
    Ok(())
}
