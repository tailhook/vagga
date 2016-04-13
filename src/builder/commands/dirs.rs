use std::fs::{create_dir_all, set_permissions, Permissions, remove_file};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::ffi::OsStrExt;
use std::collections::BTreeMap;

use std::path::{Path, PathBuf};
use path_util::ToRelative;
use container::util::{clean_dir};
use build_step::{BuildStep, VersionError, StepError, Digest, Config, Guard};


#[derive(Debug)]
pub struct EnsureDir(PathBuf);
tuple_struct_decode!(EnsureDir);

#[derive(Debug)]
pub struct Remove(PathBuf);
tuple_struct_decode!(Remove);

#[derive(Debug)]
pub struct EmptyDir(PathBuf);
tuple_struct_decode!(EmptyDir);

#[derive(Debug)]
pub struct CacheDirs(BTreeMap<PathBuf, String>);
tuple_struct_decode!(CacheDirs);


pub fn remove(path: &PathBuf, guard: &mut Guard)
    -> Result<(), StepError>
{
    let fpath = Path::new("/vagga/root").join(path.rel());
    if fpath.is_dir() {
        try!(clean_dir(&fpath, true));
    } else if fpath.exists() {
        try!(remove_file(&fpath)
            .map_err(|e| format!("Error removing file {:?}: {}",
                                 &fpath, e)));
    }
    guard.ctx.add_remove_dir(&path);
    Ok(())
}

pub fn ensure(path: &PathBuf, guard: &mut Guard)
    -> Result<(), StepError>
{
    let fpath = Path::new("/vagga/root").join(path.rel());
    try!(create_dir_all(&fpath)
        .map_err(|e| format!("Error creating dir: {}", e)));
    try!(set_permissions(&fpath, Permissions::from_mode(0o755))
        .map_err(|e| format!("Error setting permissions: {}", e)));
    for mount_point in guard.ctx.container_config.volumes.keys() {
        if path != mount_point && path.starts_with(mount_point) {
            warn!("{0:?} directory is in the volume: {1:?}.\n\t\
                   {0:?} will be unaccessible inside the container.",
                path,
                mount_point);
        }
    }
    guard.ctx.add_ensure_dir(path);
    Ok(())
}

impl BuildStep for EnsureDir {
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.field("EnsureDir", self.0.as_os_str().as_bytes());
        Ok(())
    }
    fn build(&self, guard: &mut Guard, _build: bool)
        -> Result<(), StepError>
    {
        ensure(&self.0, guard)
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for EmptyDir {
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.field("EmptyDir", self.0.as_os_str().as_bytes());
        Ok(())
    }
    fn build(&self, guard: &mut Guard, _build: bool)
        -> Result<(), StepError>
    {
        try!(clean_dir(&self.0, false));
        guard.ctx.add_empty_dir(&self.0);
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for CacheDirs {
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        for (k, v) in self.0.iter() {
            hash.field(k.as_os_str().as_bytes(), v);
        }
        Ok(())
    }
    fn build(&self, guard: &mut Guard, _build: bool)
        -> Result<(), StepError>
    {
        for (k, v) in self.0.iter() {
            try!(guard.ctx.add_cache_dir(k, v.clone()));
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for Remove {
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.field("Remove", self.0.as_os_str().as_bytes());
        Ok(())
    }
    fn build(&self, guard: &mut Guard, _build: bool)
        -> Result<(), StepError>
    {
        remove(&self.0, guard)
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}
