use std::io::ALL_PERMISSIONS;
use std::io::fs::{mkdir_recursive};
use std::io::fs::PathExtensions;
use std::io::process::{Command, Ignored, InheritFd, ExitStatus};

use container::mount::{bind_mount, unmount};
use config::builders::TarInfo;

use super::context::BuildContext;
use super::download::download_file;


pub fn unpack_file(_ctx: &mut BuildContext, src: &Path, tgt: &Path,
    includes: &[Path], excludes: &[Path])
    -> Result<(), String>
{
    info!("Unpacking {} -> {}", src.display(), tgt.display());
    let mut cmd = Command::new("/vagga/bin/busybox");
    cmd.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2))
        .arg("tar")
        .arg("-x")
        .arg("-f").arg(src)
        .arg("-C").arg(tgt);
    for i in includes.iter() {
        cmd.arg(i);
    }
    for i in excludes.iter() {
        cmd.arg("--exclude").arg(i);
    }

    match src.extension_str() {
        Some("gz")|Some("tgz") => { cmd.arg("-z"); }
        Some("bz")|Some("tbz") => { cmd.arg("-j"); }
        Some("xz")|Some("txz") => { cmd.arg("-J"); }
        _ => {}
    };
    debug!("Running: {}", cmd);
    match cmd.output()
        .map_err(|e| format!("Can't run tar: {}", e))
        .map(|o| o.status)
    {
        Ok(ExitStatus(0)) => Ok(()),
        Ok(val) => Err(format!("Tar exited with status: {}", val)),
        Err(x) => Err(format!("Error running tar: {}", x)),
    }
}

pub fn tar_command(ctx: &mut BuildContext, tar: &TarInfo) -> Result<(), String>
{
    let fpath = Path::new("/vagga/root").join(
        tar.path.path_relative_from(&Path::new("/")).unwrap());
    let filename = try!(download_file(ctx, &tar.url));
    // TODO(tailhook) check sha256 sum
    if tar.subdir == Path::new(".") {
        try!(unpack_file(ctx, &filename, &fpath, &[], &[]));
    } else {
        let tmppath = Path::new("/vagga/root/tmp")
            .join(filename.filename_str().unwrap());
        let tmpsub = tmppath.join(&tar.subdir);
        try!(mkdir_recursive(&tmpsub, ALL_PERMISSIONS)
             .map_err(|e| format!("Error making dir: {}", e)));
        if !fpath.exists() {
            try!(mkdir_recursive(&fpath, ALL_PERMISSIONS)
                 .map_err(|e| format!("Error making dir: {}", e)));
        }
        try!(bind_mount(&fpath, &tmpsub));
        let res = unpack_file(ctx, &filename, &tmppath,
            &[tar.subdir.clone()], &[]);
        try!(unmount(&tmpsub));
        try!(res);
    }
    Ok(())
}
