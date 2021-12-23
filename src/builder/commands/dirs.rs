use std::io;
use std::fs::remove_file;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use quire::validate as V;

#[cfg(feature="containers")]
use crate::{
    container::util::clean_dir,
    file_util::Dir,
};
use crate::build_step::{BuildStep, VersionError, StepError, Digest, Config, Guard};


#[derive(Debug, Serialize, Deserialize)]
pub struct EnsureDir(PathBuf);

impl EnsureDir {
    pub fn config() -> V::Directory {
        V::Directory::new().absolute(true)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Remove(PathBuf);

impl Remove {
    pub fn config() -> V::Directory {
        V::Directory::new().absolute(true)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmptyDir(PathBuf);

impl EmptyDir {
    pub fn config() -> V::Directory {
        V::Directory::new().absolute(true)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheDirs(BTreeMap<PathBuf, String>);

impl CacheDirs {
    pub fn config() -> V::Mapping<'static> {
        V::Mapping::new(
            V::Directory::new().absolute(true),
            V::Scalar::new())
    }
}

#[cfg(feature="containers")]
pub fn remove(path: &PathBuf)
    -> Result<(), StepError>
{
    let ref fpath = Path::new("/vagga/root")
        .join(path.strip_prefix("/")
            .map_err(|_| format!("Must be absolute: {:?}", path))?);
    match fpath.symlink_metadata() {
        Ok(ref stats) if stats.is_dir() => {
            clean_dir(fpath, true)?;
        },
        Ok(_) => {
            remove_file(fpath)
                 .map_err(|e| format!("Error removing file {:?}: {}",
                     fpath, e))?;
        },
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {},
        Err(_) => {
            return Err(StepError::from(format!("Cannot stat {:?}",
                path)));
        },
    }
    Ok(())
}

#[cfg(feature="containers")]
pub fn ensure(path: &PathBuf)
    -> Result<(), StepError>
{
    let ref fpath = Path::new("/vagga/root")
        .join(path.strip_prefix("/")
            .map_err(|_| format!("Must be absolute: {:?}", path))?);
    match fpath.metadata() {
        Ok(ref stats) if stats.is_dir() => {
            return Ok(());
        },
        Ok(_) => {
            return Err(StepError::from(format!(
                "Path {:?} exists but not a directory", path)));
        },
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            match Dir::new(fpath).recursive(true).create() {
                Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => {
                    return Err(StepError::from(format!(
                        "Some intermediate path for {:?} exists \
                         but not a directory", path)));
                },
                Err(_) => {
                    return Err(StepError::from(format!(
                        "Error creating dir: {}", e)));
                },
                Ok(_) => {},
            }
        },
        Err(_) => {
            return Err(StepError::from(format!("Cannot stat {:?}",
                path)));
        },
    }
    Ok(())
}

impl BuildStep for EnsureDir {
    fn name(&self) -> &'static str { "EnsureDir" }
    #[cfg(feature="containers")]
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.field("path", &self.0);
        Ok(())
    }
    #[cfg(feature="containers")]
    fn build(&self, guard: &mut Guard, _build: bool)
        -> Result<(), StepError>
    {
        let ref path = self.0;
        ensure(path)?;
        for mount_point in guard.ctx.container_config.volumes.keys() {
            if path != mount_point && path.starts_with(mount_point) {
                warn!("{0:?} directory is in the volume: {1:?}.\n\t\
                       {0:?} will be unaccessible inside the container.",
                    path,
                    mount_point);
            }
        }
        guard.ctx.add_ensure_dir(path)?;
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for EmptyDir {
    fn name(&self) -> &'static str { "EmptyDir" }
    #[cfg(feature="containers")]
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.field("path", &self.0);
        Ok(())
    }
    #[cfg(feature="containers")]
    fn build(&self, guard: &mut Guard, _build: bool)
        -> Result<(), StepError>
    {
        clean_dir(&self.0, false)?;
        guard.ctx.add_empty_dir(&self.0)?;
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for CacheDirs {
    fn name(&self) -> &'static str { "CacheDirs" }
    #[cfg(feature="containers")]
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        for (k, v) in self.0.iter() {
            hash.field("source", k);
            hash.field("name", v);
        }
        Ok(())
    }
    #[cfg(feature="containers")]
    fn build(&self, guard: &mut Guard, _build: bool)
        -> Result<(), StepError>
    {
        for (k, v) in self.0.iter() {
            guard.ctx.add_cache_dir(k, &v)?;
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for Remove {
    fn name(&self) -> &'static str { "Remove" }
    #[cfg(feature="containers")]
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.field("path", &self.0);
        Ok(())
    }
    #[cfg(feature="containers")]
    fn build(&self, guard: &mut Guard, _build: bool)
        -> Result<(), StepError>
    {
        remove(&self.0)?;
        guard.ctx.add_remove_path(&self.0)?;
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}
