use std::path::Path;
use std::fs::symlink_metadata;
use std::os::unix::fs::{MetadataExt, PermissionsExt};

use libmount::{BindMount, Tmpfs};

use config::volumes::SnapshotInfo;
use file_util::Dir;
use container::mount::{unmount};
use container::util::{copy_dir};


pub fn make_snapshot(src: &Path, dest: &Path, info: &SnapshotInfo)
    -> Result<(), String>
{
    let tmp = Path::new("/tmp/mnt");
    try_msg!(Dir::new(&tmp).recursive(true).create(),
        "Error creating temporary mountpoint: {err}");
    let stat = try_msg!(symlink_metadata(&dest),
        "Error getting mountpoint metadata: {err}");
    let mode = stat.permissions().mode();
    let (uid, gid) = (stat.uid(), stat.gid());
    BindMount::new(&src, &tmp).mount().map_err(|e| e.to_string())?;
    Tmpfs::new(&dest)
        .size_bytes(info.size)
        .mode(mode)
        .uid(uid)
        .gid(gid)
        .mount().map_err(|e| format!("{}", e))?;
    try_msg!(copy_dir(&tmp, dest, info.owner_uid, info.owner_gid),
        "Error copying directory: {err}");
    unmount(tmp)?;
    Ok(())
}
