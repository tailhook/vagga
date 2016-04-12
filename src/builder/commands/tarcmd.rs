use std::fs::{create_dir_all, set_permissions, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use unshare::{Command, Stdio};
use libmount::BindMount;

use container::mount::{unmount};
use builder::context::Context;
use builder::download::download_file;
use builder::commands::generic::run_command_at;
use file_util::{read_visible_entries, create_dir};
use path_util::ToRelative;
use build_step::{BuildStep, VersionError, StepError, Digest, Config, Guard};


#[derive(RustcDecodable, Debug)]
pub struct Tar {
    pub url: String,
    pub sha256: Option<String>,
    pub path: PathBuf,
    pub subdir: PathBuf,
}

#[derive(RustcDecodable, Debug)]
pub struct TarInstall {
    pub url: String,
    pub sha256: Option<String>,
    pub subdir: Option<PathBuf>,
    pub script: String,
}


pub fn unpack_file(_ctx: &mut Context, src: &Path, tgt: &Path,
    includes: &[&Path], excludes: &[&Path])
    -> Result<(), String>
{
    info!("Unpacking {} -> {}", src.display(), tgt.display());
    let mut cmd = Command::new("/vagga/bin/busybox");
    cmd.stdin(Stdio::null())
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

    match src.extension().and_then(|x| x.to_str()) {
        Some("gz")|Some("tgz") => { cmd.arg("-z"); }
        Some("bz")|Some("tbz") => { cmd.arg("-j"); }
        Some("xz")|Some("txz") => { cmd.arg("-J"); }
        _ => {}
    };
    info!("Running: {:?}", cmd);
    match cmd.status() {
        Ok(st) if st.success() => Ok(()),
        Ok(val) => Err(format!("Tar exited with status: {}", val)),
        Err(e) => Err(format!("Error running tar: {}", e)),
    }
}

pub fn tar_command(ctx: &mut Context, tar: &Tar) -> Result<(), String>
{
    let fpath = PathBuf::from("/vagga/root").join(tar.path.rel());
    let filename = if tar.url.starts_with(".") {
        PathBuf::from("/work").join(&tar.url)
    } else {
        try!(download_file(ctx, &tar.url))
    };
    // TODO(tailhook) check sha256 sum
    if &Path::new(&tar.subdir) == &Path::new(".") {
        try!(unpack_file(ctx, &filename, &fpath, &[], &[]));
    } else {
        let tmppath = PathBuf::from("/vagga/root/tmp")
            .join(filename.file_name().unwrap());
        let tmpsub = tmppath.join(&tar.subdir);
        try_msg!(create_dir(&tmpsub, true), "Error making dir: {err}");
        if !fpath.exists() {
            try_msg!(create_dir(&fpath, true), "Error making dir: {err}");
        }
        try_msg!(BindMount::new(&fpath, &tmpsub).mount(),
            "temporary tar mount: {err}");
        let res = if tar.subdir.as_path() == Path::new("") {
            unpack_file(ctx, &filename, &tmppath, &[], &[])
        } else {
            unpack_file(ctx, &filename, &tmppath,
                &[&tar.subdir.clone()], &[])
        };
        try!(unmount(&tmpsub));
        try!(res);
    }
    Ok(())
}

pub fn tar_install(ctx: &mut Context, tar: &TarInstall)
    -> Result<(), String>
{
    let filename = if tar.url.starts_with(".") {
        PathBuf::from("/work").join(&tar.url)
    } else {
        try!(download_file(ctx, &tar.url))
    };
    // TODO(tailhook) check sha256 sum
    let tmppath = PathBuf::from("/vagga/root/tmp")
        .join(filename.file_name().unwrap());
    try!(create_dir_all(&tmppath)
         .map_err(|e| format!("Error making dir: {}", e)));
    try!(set_permissions(&tmppath, Permissions::from_mode(0o755))
         .map_err(|e| format!("Error setting permissions: {}", e)));
    try!(unpack_file(ctx, &filename, &tmppath, &[], &[]));
    let workdir = if let Some(ref subpath) = tar.subdir {
        tmppath.join(subpath)
    } else {
        let items = try!(read_visible_entries(&tmppath)
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
    let workdir = PathBuf::from("/").join(
        workdir.rel_to(&Path::new("/vagga/root")).unwrap());
    return run_command_at(ctx, &[
        "/bin/sh".to_string(),
        "-exc".to_string(),
        tar.script.to_string()],
        &workdir);
}

impl BuildStep for Tar {
    fn hash(&self, cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        if let Some(ref sha) = self.sha256 {
            hash.field("hash", sha);
        } else {
            hash.field("url", &self.url);
        }
        hash.field("path", self.path.as_os_str().as_bytes());
        hash.field("subdir", self.subdir.as_os_str().as_bytes());
        Ok(())
    }
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        if build {
            try!(tar_command(&mut guard.ctx, self));
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for TarInstall {
    fn hash(&self, cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        if let Some(ref sha) = self.sha256 {
            hash.field("hash", sha);
        } else {
            hash.field("url", &self.url);
        }
        hash.opt_field("subdir",
            &self.subdir.as_ref().map(|x| x.as_os_str().as_bytes()));
        hash.field("script", &self.script);
        Ok(())
    }
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        if build {
            try!(tar_install(&mut guard.ctx, self));
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}
