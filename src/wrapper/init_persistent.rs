use std::io;
use std::path::Path;
use std::os::unix::io::RawFd;

use nix::fcntl;
use libc::{open, renameat};

use crate::config::volumes::PersistentInfo;
use crate::file_util::{Lock, Dir};
use crate::container::util::clean_dir;
use crate::path_util::ToCString;


pub trait Guard {
    fn commit(&self) -> Result<(), String>;
}

pub struct PersistentVolumeGuard {
    #[allow(dead_code)]  // we store lock here to keep it alive
    lock: Lock,
    volumes_base: RawFd,
    volume_name: String,
}

impl PersistentVolumeGuard {
    #[cfg(feature="containers")]
    pub fn new(vol: &PersistentInfo)
        -> io::Result<Option<PersistentVolumeGuard>>
    {
        let volbase = Path::new("/vagga/base/.volumes");
        let path = volbase.join(&vol.name);
        if path.exists() {
            return Ok(None);
        }
        Dir::new(&volbase).create()?;
        let lockfile = volbase.join(format!(".tmp.{}.lock", vol.name));
        let lock = Lock::exclusive(lockfile)?;
        if path.exists() {
            return Ok(None);
        }
        let tmpdir = volbase.join(format!(".tmp.{}", vol.name));
        if tmpdir.exists() {
            clean_dir(&tmpdir, false)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        }
        Dir::new(&tmpdir)
            .uid(vol.owner_uid)
            .gid(vol.owner_gid)
            .create()?;
        let volumes_base = unsafe {
            open(volbase.to_cstring().as_ptr(), fcntl::OFlag::O_PATH.bits())
        };
        if volumes_base < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Some(PersistentVolumeGuard {
            lock: lock,
            volumes_base: volumes_base,
            volume_name: vol.name.clone(),
        }))
    }
    #[cfg(not(feature="containers"))]
    pub fn new(vol: &PersistentInfo)
        -> io::Result<Option<PersistentVolumeGuard>>
    {
        unimplemented!()
    }
}

impl Guard for PersistentVolumeGuard {
    fn commit(&self) -> Result<(), String> {
        let rc = unsafe {
            renameat(self.volumes_base,
                format!(".tmp.{}", self.volume_name).to_cstring().as_ptr(),
                self.volumes_base, self.volume_name.to_cstring().as_ptr())
        };
        if rc < 0 {
            return Err(format!("Error commiting volume: {}",
                               io::Error::last_os_error()));
        }
        Ok(())
    }
}
