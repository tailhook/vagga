use std::path::Path;
use std::fs::symlink_metadata;
use std::os::unix::fs::{MetadataExt, PermissionsExt};

use libmount::{BindMount, Tmpfs};

use config::volumes::SnapshotInfo;
use file_util::create_dir;
use container::mount::{unmount};
use container::util::{copy_dir};


pub fn make_snapshot(dest: &Path, info: &SnapshotInfo)
    -> Result<(), String>
{
    let tmp = Path::new("/tmp/mnt");
    try_msg!(create_dir(&tmp, true),
        "Error creating temporary mountpoint: {err}");
    let stat = try_msg!(symlink_metadata(&dest),
        "Error getting mountpoint metadata: {err}");
    let mode = stat.permissions().mode();
    let (uid, gid) = (stat.uid(), stat.gid());
    try!(BindMount::new(&dest, &tmp).mount().map_err(|e| e.to_string()));
    try!(Tmpfs::new(&dest)
        .size_bytes(info.size)
        .mode(mode)
        .uid(uid) 
        .gid(gid)
        .mount().map_err(|e| format!("{}", e)));
    try_msg!(copy_dir(&tmp, dest, info.owner_uid, info.owner_gid),
        "Error copying directory: {err}");
    try!(unmount(tmp));
    Ok(())
}
