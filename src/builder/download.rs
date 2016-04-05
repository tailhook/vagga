use std::fs::{remove_file, rename, create_dir_all, set_permissions};
use std::fs::{File, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use shaman::digest::Digest;
use shaman::sha2::Sha256;
use unshare::{Command, Stdio};

use super::capsule;
use super::context::Context;
use file_util::check_stream_hashsum;


pub fn download_file(ctx: &mut Context, url: &str, sha256: Option<String>)
    -> Result<PathBuf, String>
{
    let https = url.starts_with("https:");
    if https {
        try!(capsule::ensure_features(ctx, &[capsule::Https]));
    }
    let urlpath = Path::new(url);
    let hash = match sha256 {
        Some(ref sha256) => sha256[..8].to_string(),
        None => {
            let mut hash = Sha256::new();
            hash.input_str(url);
            hash.result_str()[..8].to_string()
        },
    };
    let name = match urlpath.file_name().and_then(|x| x.to_str()) {
        Some(name) => name,
        None => "file.bin",
    };
    let name = hash[..8].to_string() + "-" + name;
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
            if let Some(ref sha256) = sha256 {
                let mut tmpfile = try_msg!(File::open(&tmpfilename),
                                        "Cannot open archive: {err}");
                if let Err(e) = check_stream_hashsum(&mut tmpfile, sha256) {
                    remove_file(&filename)
                        .map_err(|e| error!("Error unlinking cache file: {}", e)).ok();
                    return Err(e);
                }
            }
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

pub fn maybe_download_and_check_hashsum(ctx: &mut Context,
    url: &str, sha256: Option<String>)
    -> Result<PathBuf, String>
{
    let filename = if url.starts_with(".") {
        let filename = PathBuf::from("/work").join(url);
        if let Some(ref sha256) = sha256 {
            let mut file = try_msg!(File::open(&filename),
                "Cannot open file: {err}");
            try!(check_stream_hashsum(&mut file, sha256)
                .map_err(|e| format!(
                    "Error when checking hashsum for file {:?}: {}", &filename, e)));
        }
        filename
    } else {
        try!(download_file(ctx, url, sha256))
    };

    Ok(filename)
}
