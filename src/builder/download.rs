use std::fs::{remove_file, rename, create_dir_all, set_permissions};
use std::fs::{Permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use shaman::digest::Digest;
use shaman::sha2::Sha256;
use unshare::{Command, Stdio};

use super::capsule;
use super::context::Context;


pub fn download_file(ctx: &mut Context, url: &str)
    -> Result<PathBuf, String>
{
    let https = url.starts_with("https:");
    if https {
        try!(capsule::ensure_features(ctx, &[capsule::Https]));
    }
    let mut hash = Sha256::new();
    hash.input_str(url);
    let urlpath = Path::new(url);
    let name = match urlpath.file_name().and_then(|x| x.to_str()) {
        Some(name) => name,
        None => "file.bin",
    };
    let name = hash.result_str()[..8].to_string() + "-" + name;
    let dir = Path::new("/vagga/cache/downloads");
    if !dir.exists() {
        try!(create_dir_all(&dir)
            .map_err(|e| format!("Error moving file: {}", e)));
        try!(set_permissions(&dir, Permissions::from_mode(0o755))
            .map_err(|e| format!("Can't chmod file: {}", e)));
    }
    let filename = dir.join(&name);
    if filename.exists() {
        return Ok(filename);
    }
    info!("Downloading image {} -> {}", url, filename.display());
    let tmpfilename = filename.with_file_name(name + ".part");
    let mut cmd = Command::new(
        if https { "/usr/bin/wget" } else { "/vagga/bin/busybox" });
    cmd.stdin(Stdio::null());
    if !https {
        cmd.arg("wget");
    }
    cmd.arg("-O");
    cmd.arg(&tmpfilename);
    cmd.arg(url);

    debug!("Running: {:?}", cmd);
    match cmd.status() {
        Ok(st) if st.success() => {
            try!(rename(&tmpfilename, &filename)
                .map_err(|e| format!("Error moving file: {}", e)));
            Ok(filename)
        }
        Ok(val) => {
            remove_file(&tmpfilename)
                .map_err(|e| error!("Error unlinking cache file: {}", e)).ok();
            Err(format!("Wget exited with status: {}", val))
        }
        Err(x) => {
            remove_file(&tmpfilename)
                .map_err(|e| error!("Error unlinking cache file: {}", e)).ok();
            Err(format!("Error starting wget: {}", x))
        }
    }
}
