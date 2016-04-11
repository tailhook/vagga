use std::fs::rename;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

use unshare::{Command, Stdio};

use config::settings::Settings;
use config::builders::Git;
use config::builders::GitInstall;
use config::builders::GitSource;
use super::super::capsule;
use super::super::context::Context;
use super::generic::run_command_at;
use super::super::super::path_util::ToRelative;


fn git_cache(url: &String) -> Result<PathBuf, String> {
    let dirname = url.replace("%", "%25").replace("/", "%2F");
    let cache_path = Path::new("/vagga/cache/git").join(&dirname);
    if cache_path.is_dir() {
        let mut cmd = Command::new("/usr/bin/git");
        cmd.stdin(Stdio::null());
        cmd.arg("-C").arg(&cache_path);
        cmd.arg("fetch");
        cmd.current_dir(&cache_path);
        match cmd.status() {
            Ok(ref st) if st.success() => {}
            Ok(status) => {
                return Err(format!("Command {:?} exited with code  {}",
                    cmd, status));
            }
            Err(err) => {
                return Err(format!("Error running {:?}: {}", cmd, err));
            }
        }
    } else {
        let tmppath = cache_path.with_file_name(".tmp.".to_string() + &dirname);
        let mut cmd = Command::new("/usr/bin/git");
        cmd.stdin(Stdio::null());
        cmd.arg("clone").arg("--bare");
        cmd.arg(url).arg(&tmppath);
        match cmd.status() {
            Ok(ref st) if st.success() => {}
            Ok(status) => {
                return Err(format!("Command {:?} exited with code  {}",
                    cmd, status));
            }
            Err(err) => {
                return Err(format!("Error running {:?}: {}", cmd, err));
            }
        }
        try!(rename(&tmppath, &cache_path)
            .map_err(|e| format!("Error renaming cache dir: {}", e)));
    }
    Ok(cache_path)
}

fn git_checkout(cache_path: &Path, dest: &Path,
    revision: &Option<String>, branch: &Option<String>)
    -> Result<(), String>
{
    let mut cmd = Command::new("/usr/bin/git");
    cmd.stdin(Stdio::null());
    cmd.arg("--git-dir").arg(cache_path);
    cmd.arg("--work-tree").arg(dest);
    cmd.arg("reset").arg("--hard");
    if let &Some(ref rev) = revision {
        cmd.arg(&rev);
    } else if let &Some(ref branch) = branch {
        cmd.arg(&branch);
    } else {
    }
    match cmd.status() {
        Ok(ref st) if st.success() => {}
        Ok(status) => {
            return Err(format!("Command {:?} exited with code  {}",
                cmd, status));
        }
        Err(err) => {
            return Err(format!("Error running {:?}: {}", cmd, err));
        }
    }
    Ok(())
}


pub fn git_command(ctx: &mut Context, git: &Git) -> Result<(), String>
{
    try!(capsule::ensure_features(ctx, &[capsule::Git]));
    let dest = PathBuf::from("/vagga/root").join(&git.path.rel());
    let cache_path = try!(git_cache(&git.url));
    try!(create_dir_all(&dest)
         .map_err(|e| format!("Error creating dir: {}", e)));
    try!(git_checkout(&cache_path, &dest, &git.revision, &git.branch));
    Ok(())
}

pub fn git_install(ctx: &mut Context, git: &GitInstall)
    -> Result<(), String>
{
    try!(capsule::ensure_features(ctx, &[capsule::Git]));
    let cache_path = try!(git_cache(&git.url));
    let tmppath = Path::new("/vagga/root/tmp")
        .join(cache_path.file_name().unwrap());
    try!(create_dir_all(&tmppath)
         .map_err(|e| format!("Error creating dir: {}", e)));
    try!(git_checkout(&cache_path, &tmppath, &git.revision, &git.branch));
    let workdir = PathBuf::from("/")
        .join(tmppath.rel_to(&Path::new("/vagga/root")).unwrap())
        .join(&git.subdir);
    return run_command_at(ctx, &[
        "/bin/sh".to_string(),
        "-exc".to_string(),
        git.script.to_string()],
        &workdir);
}

#[allow(unused)]
pub fn fetch_git_source(capsule: &mut capsule::State, settings: &Settings,
    git: &GitSource)
    -> Result<(), String>
{
    try!(capsule::ensure(capsule, settings, &[capsule::Git]));
    let cache_path = try!(git_cache(&git.url));
    let dest = Path::new("/vagga/sources")
        .join(cache_path.file_name().unwrap());
    try!(create_dir_all(&dest)
         .map_err(|e| format!("Error creating dir: {}", e)));
    try!(git_checkout(&cache_path, &dest, &git.revision, &git.branch));
    Ok(())
}
