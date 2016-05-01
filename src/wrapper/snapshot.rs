use std::path::Path;

use libmount::{BindMount, Tmpfs};

use config::containers::SnapshotInfo;
use file_util::create_dir;
use container::mount::{unmount};
use container::util::{copy_dir};


pub fn make_snapshot(dest: &Path, info: &SnapshotInfo)
    -> Result<(), String>
{
    let tmp = Path::new("/tmp/mnt");
    try_msg!(create_dir(&tmp, true),
        "Error creating temporary mountpoint: {err}");
    try!(BindMount::new(&dest, &tmp).mount().map_err(|e| e.to_string()));
    try!(Tmpfs::new(&dest)
        .size_bytes(info.size)
        .mode(0o755)
        .mount().map_err(|e| format!("{}", e)));
    try_msg!(copy_dir(&tmp, dest, info.owner_uid, info.owner_gid),
        "Error copying directory: {err}");
    try!(unmount(tmp));
    Ok(())
}
