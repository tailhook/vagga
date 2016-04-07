use std::path::Path;

use libmount::BindMount;

use config::containers::SnapshotInfo;
use file_util::create_dir;
use container::mount::{mount_tmpfs, unmount};
use container::util::{copy_dir};


pub fn make_snapshot(dest: &Path, info: &SnapshotInfo)
    -> Result<(), String>
{
    let tmp = Path::new("/tmp/mnt");
    try_msg!(create_dir(&tmp, true),
        "Error creating temporary mountpoint: {err}");
    try!(BindMount::new(&dest, &tmp).mount().map_err(|e| e.to_string()));
    try!(mount_tmpfs(&dest,
        &format!("size={},mode=0755", info.size)));
    try_msg!(copy_dir(&tmp, dest, info.owner_uid, info.owner_gid),
        "Error copying directory: {err}");
    try!(unmount(tmp));
    Ok(())
}
