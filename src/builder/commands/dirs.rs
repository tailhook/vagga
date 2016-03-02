use std::fs::{create_dir_all, set_permissions, Permissions, remove_file};
use std::os::unix::fs::PermissionsExt;

use std::path::{Path, PathBuf};
use path_util::ToRelative;
use builder::error::StepError;
use container::util::{clean_dir};
use builder::guard::Guard;


pub fn remove(path: &PathBuf, guard: &mut Guard)
    -> Result<(), StepError>
{
    let fpath = Path::new("/vagga/root").join(path.rel());
    if fpath.is_dir() {
        try!(clean_dir(&fpath, true));
    } else if fpath.exists() {
        try!(remove_file(&fpath)
            .map_err(|e| format!("Error removing file {:?}: {}",
                                 &fpath, e)));
    }
    guard.ctx.add_remove_dir(&path);
    Ok(())
}

pub fn ensure(path: &PathBuf, guard: &mut Guard)
    -> Result<(), StepError>
{
    let fpath = Path::new("/vagga/root").join(path.rel());
    try!(create_dir_all(&fpath)
        .map_err(|e| format!("Error creating dir: {}", e)));
    try!(set_permissions(&fpath, Permissions::from_mode(0o755))
        .map_err(|e| format!("Error setting permissions: {}", e)));
    for mount_point in guard.ctx.container_config.volumes.keys() {
        if path != mount_point && path.starts_with(mount_point) {
            warn!("{0:?} directory is in the volume: {1:?}.\n\t\
                   {0:?} will be unaccessible inside the container.",
                path,
                mount_point);
        }
    }
    guard.ctx.add_ensure_dir(path);
    Ok(())
}
