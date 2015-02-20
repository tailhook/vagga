use std::io::ALL_PERMISSIONS;
use std::io::fs::PathExtensions;
use std::io::fs::{unlink, rename, mkdir_recursive};
use std::io::process::{Command, Ignored, InheritFd, ExitStatus};

use container::sha256::{Sha256, Digest};

use super::capsule;
use super::context::BuildContext;


pub fn download_file(ctx: &mut BuildContext, url: &str)
    -> Result<Path, String>
{
    let https = url.starts_with("https:");
    if https {
        try!(capsule::ensure_features(ctx, &[capsule::Https]));
    }
    let mut hash = Sha256::new();
    hash.input_str(url);
    let urlpath = Path::new(url);
    let name = match urlpath.filename_str() {
        Some(name) => name,
        None => "file.bin",
    };
    let name = hash.result_str()[..8].to_string() + "-" + name;
    let dir = Path::new("/vagga/cache/downloads");
    if !dir.exists() {
        try!(mkdir_recursive(&dir, ALL_PERMISSIONS)
            .map_err(|e| format!("Error moving file: {}", e)));
    }
    let filename = dir.join(name.as_slice());
    if filename.exists() {
        return Ok(filename);
    }
    info!("Downloading image {} -> {}", url, filename.display());
    let tmpfilename = filename.with_filename(name + ".part");
    let mut cmd = Command::new(
        if https { "/usr/bin/wget" } else { "/vagga/bin/busybox" });
    cmd.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2));
    if !https {
        cmd.arg("wget");
    }
    cmd.arg("-O");
    cmd.arg(&tmpfilename);
    cmd.arg(url);
    debug!("Running: {}", cmd);
    match cmd
        .output()
        .map_err(|e| format!("Can't run wget: {}", e))
        .map(|o| o.status)
    {
        Ok(ExitStatus(0)) => {
            try!(rename(&tmpfilename, &filename)
                .map_err(|e| format!("Error moving file: {}", e)));
            Ok(filename)
        }
        Ok(val) => {
            unlink(&tmpfilename)
                .map_err(|e| error!("Error unlinking cache file: {}", e)).ok();
            Err(format!("Wget exited with status: {}", val))
        }
        Err(x) => {
            unlink(&tmpfilename)
                .map_err(|e| error!("Error unlinking cache file: {}", e)).ok();
            Err(x)
        }
    }
}
