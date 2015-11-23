use std::path::{PathBuf};
use std::fs::{set_permissions, Permissions};
use std::os::unix::fs::PermissionsExt;

use config::builders::DownloadInfo;
use file_util::copy;
use path_util::ToRelative;
use builder::context::Context;
use builder::download::download_file;


pub fn download(ctx: &mut Context, dlinfo: &DownloadInfo) -> Result<(), String>
{
    let fpath = PathBuf::from("/vagga/root").join(dlinfo.path.rel());
    let filename = if dlinfo.url.starts_with(".") {
        PathBuf::from("/work").join(&dlinfo.url)
    } else {
        try!(download_file(ctx, &dlinfo.url))
    };
    try!(copy(&filename, &fpath)
        .map_err(|e| format!("Error copying {:?} to {:?}: {}",
            &filename, dlinfo.path, e)));
    try!(set_permissions(&fpath, Permissions::from_mode(dlinfo.mode))
        .map_err(|e| format!("Error setting permissions for {:?}: {}",
            dlinfo.path, e)));
    Ok(())
}
