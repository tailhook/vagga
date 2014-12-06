use std::io::ALL_PERMISSIONS;
use std::io::fs::PathExtensions;
use std::io::fs::{unlink, rename, mkdir_recursive};
use std::io::process::{Command, Ignored, InheritFd, ExitStatus};

use container::sha256::{Sha256, Digest};

use super::context::BuildContext;


pub fn unpack_file(ctx: &mut BuildContext, src: &Path, tgt: &Path)
    -> Result<(), String>
{
    info!("Unpacking {} -> {}", src.display(), tgt.display());
    match Command::new("/vagga/bin/busybox")
            .stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2))
            .arg("tar")
            .arg("-x")
            .arg("-f").arg(src)
            .arg("-C").arg(tgt)
        .output()
        .map_err(|e| format!("Can't run tar: {}", e))
        .map(|o| o.status)
    {
        Ok(ExitStatus(0)) => Ok(()),
        Ok(val) => Err(format!("Tar exited with status: {}", val)),
        Err(x) => Err(format!("Error running tar: {}", x)),
    }
}
