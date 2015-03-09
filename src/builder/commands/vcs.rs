use std::process::{Command, Stdio};
use std::fs::rename;
use std::old_io::fs::PathExtensions;
use std::fs::create_dir_all;

use config::builders::GitInfo;
use config::builders::GitInstallInfo;

use super::super::capsule;
use super::super::context::BuildContext;


pub fn git_command(ctx: &mut BuildContext, git: &GitInfo) -> Result<(), String>
{
    try!(capsule::ensure_features(ctx, &[capsule::Git]));
    let fpath = Path::new("/vagga/root").join(
        git.path.path_relative_from(&Path::new("/")).unwrap());
    let dirname = git.url.replace("%", "%25").replace("/", "%2F");
    let cache_path = Path::new("/vagga/cache/git").join(
        git.url.replace("%", "%25").replace("/", "%2F"));
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
        let tmppath = cache_path.with_filename(".tmp.".to_string() + &dirname);
        let mut cmd = Command::new("/usr/bin/git");
        cmd.stdin(Stdio::null());
        cmd.arg("clone").arg("--bare");
        cmd.arg(&git.url).arg(&tmppath);
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
    let dest = Path::new("/vagga/root")
        .join(&git.path.path_relative_from(&Path::new("/")).unwrap());
    try!(create_dir_all(&dest)
         .map_err(|e| format!("Error creating dir: {}", e)));
    let mut cmd = Command::new("/usr/bin/git");
    cmd.stdin(Stdio::null());
    cmd.arg("--git-dir").arg(&cache_path);
    cmd.arg("--work-tree").arg(&dest);
    cmd.arg("reset").arg("--hard");
    if let Some(ref rev) = git.revision {
        cmd.arg(&rev);
    } else if let Some(ref branch) = git.revision {
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

pub fn git_install(ctx: &mut BuildContext, git: &GitInstallInfo)
    -> Result<(), String>
{
    unimplemented!();
}
