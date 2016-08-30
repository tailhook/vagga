use std::io;

use libc::{open, renameat, O_PATH};

use file_util::Lock;
use path_util::ToCString;


pub trait Guard {
    fn commit() -> Result<(), String>;
}

pub struct PersistentVolumeGuard {
    lock: Lock,
    volumes_base: RawFd,
    volume_name: String,
}

impl PersistentVolumeGuard {
    pub fn new(name: String) -> io::Result<Option<PersistentVolumeGuard>> {
        let path = Path::new("/vagga/base/.volumes").join(name);
        if path.exists() {
            return Ok(None);
        }
        let lockfile = Path::new("/vagga/base/.volumes")
            .join(format!(".tmp.{}.lock", name));
        let lock = Lock::exclusive(lockfile);
        if path.exists() {
            return Ok(None);
        }
        let tmpdir = Path::new("/vagga/base/.volumes")
            .join();
        let volumes_base = unsafe {
            open(tmpdir.to_cstring().as_ptr(), O_PATH)
        };
        if volumes_base < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Some(PersistentVolumeGuard {
            lock: lockfile,
            volumes_base: volumes_base,
            volume_name: name,
        }))
    }
}

impl Guard for PersistentVolumeGuard {
    pub fn commit(&self) -> Result<(), String> {
        let rc = unsafe {
            renameat(self.volumes_base,
                format!(".tmp.{}", name).to_cstring().as_ptr(),
                self.volumes_base, name.to_cstring().as_ptr())
        };
        if volumes_base < 0 {
            return Err(format!("Error commiting volume: {}",
                               io::Error::last_os_error()));
        }
        Ok(())
    }
}
