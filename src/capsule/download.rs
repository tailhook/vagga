use std::fs::{File, Permissions};
use std::fs::{remove_file, rename, create_dir_all, set_permissions};
use std::io::{stdout, stderr};
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use argparse::{ArgumentParser};
use argparse::{Store, StoreOption, StoreTrue};
use digest_traits::Digest;
use sha2::Sha256;
use unshare::{Command, Stdio};

use crate::capsule::{Context, packages as capsule};
use crate::capsule::packages::State;
use crate::digest::hex;
use crate::file_util::{check_stream_hashsum, Lock};
use crate::process_util::cmd_show;


pub fn download_file<S>(state: &mut State, urls: &[S], sha256: Option<String>,
    refresh: bool)
    -> Result<PathBuf, String>
    where S: AsRef<str>
{
    let urlpath = Path::new(urls[0].as_ref());
    let hash = match sha256 {
        Some(ref sha256) => sha256[..8].to_string(),
        None => {
            let mut hash = Sha256::new();
            hash.update(urls[0].as_ref().as_bytes());
            format!("{:.8x}", hex(&hash))
        },
    };
    let name = match urlpath.file_name().and_then(|x| x.to_str()) {
        Some(name) => name,
        None => "file.bin",
    };
    let name = hash[..8].to_string() + "-" + name;
    let dir = Path::new("/vagga/cache/downloads");
    if !dir.exists() {
        create_dir_all(&dir)
            .map_err(|e| format!("Error moving file: {}", e))?;
        set_permissions(&dir, Permissions::from_mode(0o755))
            .map_err(|e| format!("Can't chmod file: {}", e))?;
    }
    let filename = dir.join(&name);
    if !refresh && filename.exists() {
        return Ok(filename);
    }

    let lockfile = dir.join(&format!("{}.lock", &name));
    {
        let _lock = Lock::exclusive_wait(
            &lockfile,
            &format!("Another process are downloading the file: {:?}. Waiting", lockfile)
        )
            .map_err(|e| format!("Error when waiting a lock to download {:?}: {}", urlpath, e));
        if filename.exists() {
            return Ok(filename);
        }

        let https = urls.iter().any(|x| x.as_ref().starts_with("https:"));
        if https {
            capsule::ensure(state, &[capsule::Https])?;
        }
        for url in urls {
            let url = url.as_ref();
            info!("Downloading image {} -> {:?}", url, filename);
            let tmpfilename = filename.with_file_name(name.clone() + ".part");
            let mut cmd = Command::new(
                if https { "/usr/bin/wget" } else { "/vagga/bin/busybox" });
            cmd.stdin(Stdio::null());
            if !https {
                cmd.arg("wget");
            }
            cmd.arg("-O");
            cmd.arg(&tmpfilename);
            cmd.arg(url);

            debug!("Running: {}", cmd_show(&cmd));
            match cmd.status() {
                Ok(st) if st.success() => {
                    if let Some(ref sha256) = sha256 {
                        let mut tmpfile = try_msg!(File::open(&tmpfilename),
                                            "Cannot open archive: {err}");
                        if let Err(e) = check_stream_hashsum(&mut tmpfile, sha256) {
                            remove_file(&filename)
                                .map_err(|e| error!(
                                "Error unlinking cache file: {}", e)).ok();
                            error!("Bad hashsum of {:?}", url);
                            return Err(e);
                        }
                    }
                    rename(&tmpfilename, &filename)
                        .map_err(|e| format!("Error moving file: {}", e))?;
                    return Ok(filename);
                }
                Ok(val) => {
                    match remove_file(&tmpfilename) {
                        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                            // Assume we got 404 and download have not even started
                            continue;
                        }
                        Err(e) => {
                            error!("Error unlinking cache file: {}", e);
                        }
                        Ok(_) => {}
                    }
                    error!("Error downloading {:?}, wget {:?}", url, val);
                    continue;
                }
                Err(x) => {
                    return Err(format!("Error starting wget: {}", x));
                }
            }
        }
    }

    match remove_file(&lockfile) {
        Ok(_) => {}
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(format!("Error when removing lock file: {}", e))
    }

    return Err(format!("Error downloading file {:?} from {:?}",
        filename, urls.iter().map(|x| x.as_ref()).collect::<Vec<_>>()));
}

fn check_if_local(url: &str, sha256: &Option<String>)
    -> Result<Option<PathBuf>, String>
{
    let path = if url.starts_with(".") {
        PathBuf::from("/work").join(url)
    } else if url.starts_with("/volumes/") {
        PathBuf::from(url)
    } else {
        return Ok(None);
    };
    if let Some(ref sha256) = *sha256 {
        let mut file = try_msg!(File::open(&path),
            "Cannot open file: {err}");
        check_stream_hashsum(&mut file, sha256)
            .map_err(|e| format!(
                "Error when checking hashsum for file {:?}: {}",
                &path, e))?;
    }
    Ok(Some(path))
}

pub fn maybe_download_and_check_hashsum(state: &mut State,
    url: &str, sha256: Option<String>, refresh: bool)
    -> Result<(PathBuf, bool), String>
{
    Ok(match check_if_local(url, &sha256)? {
        Some(path) => (path, false),
        None => (download_file(state, &[url], sha256, refresh)?, true),
    })
}

pub fn run_download(context: &Context, mut args: Vec<String>)
    -> Result<i32, String>
{
    let mut url = "".to_string();
    let mut sha256 = None;
    let mut refresh = false;
    {
        args.insert(0, "vagga _capsule download".to_string());
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Downloads file if not cached, puts it into a cache printing the
            path to the cached item to the stdout.
            ");
        ap.refer(&mut sha256)
            .add_option(&["--sha256"], StoreOption,
                "A SHA256 hashsum of a file (if you want to check)");
        ap.refer(&mut refresh)
            .add_option(&["--refresh"], StoreTrue,
                "Download file even if there is a cached item");
        ap.refer(&mut url)
            .add_argument("url", Store,
                "A file to download")
            .required();
        ap.stop_on_first_argument(true);
        match ap.parse(args.clone(), &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => {
                return Ok(122);
            }
        }
    }
    // TODO(tailhook) wrap settings into Arc in the launcher's main
    let mut capsule = State::new(&Arc::new(context.settings.clone()));
    let (path, _) = maybe_download_and_check_hashsum(
        &mut capsule, &url, sha256, refresh)?;
    println!("{}", path.display());
    return Ok(0);
}
