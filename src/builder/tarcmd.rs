use std::fs::{create_dir_all, read_dir};
use std::process::{Command, ExitStatus};

use container::mount::{bind_mount, unmount};
use config::builders::TarInfo;
use config::builders::TarInstallInfo;

use super::context::BuildContext;
use super::download::download_file;
use super::commands::generic::run_command_at;


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
    debug!("Running: {:?}", cmd);
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
    let filename = try!(download_file(ctx, &tar.url[0..]));
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

pub fn tar_install(ctx: &mut BuildContext, tar: &TarInstallInfo)
    -> Result<(), String>
{
    let filename = try!(download_file(ctx, &tar.url[0..]));
    // TODO(tailhook) check sha256 sum
    let tmppath = Path::new("/vagga/root/tmp")
        .join(filename.filename_str().unwrap());
    try!(mkdir_recursive(&tmppath, ALL_PERMISSIONS)
         .map_err(|e| format!("Error making dir: {}", e)));
    try!(unpack_file(ctx, &filename, &tmppath, &[], &[]));
    let workdir = if let Some(ref subpath) = tar.subdir {
        tmppath.join(subpath)
    } else {
        let items = try!(readdir(&tmppath)
            .map_err(|e| format!("Error reading dir: {}", e)));
        if items.len() != 1 {
            if items.len() == 0 {
                return Err("Tar archive was empty".to_string());
            } else {
                return Err("Multiple directories was unpacked. \
                    If thats expected use `subdir: \".\"` or any \
                    other directory".to_string());
            }
        }
        items.into_iter().next().unwrap()
    };
    let workdir = Path::new("/").join(
        workdir.path_relative_from(&Path::new("/vagga/root")).unwrap());
    return run_command_at(ctx, &[
        "/bin/sh".to_string(),
        "-exc".to_string(),
        tar.script.to_string()],
        &workdir);
}
