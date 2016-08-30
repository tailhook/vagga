use std::io;
use std::path::Path;
use std::os::unix::io::RawFd;

use nix::fcntl::O_PATH;
use libc::{open, renameat};

use file_util::{Lock, create_dir};
use container::util::clean_dir;
use path_util::ToCString;


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
    pub fn new(name: String) -> io::Result<Option<PersistentVolumeGuard>> {
        let volbase = Path::new("/vagga/base/.volumes");
        let path = volbase.join(&name);
        if path.exists() {
            return Ok(None);
        }
        try!(create_dir(&volbase, false));
        let lockfile = volbase.join(format!(".tmp.{}.lock", name));
        let lock = try!(Lock::exclusive(lockfile));
        if path.exists() {
            return Ok(None);
        }
        let tmpdir = volbase.join(format!(".tmp.{}", name));
        if tmpdir.exists() {
            try!(clean_dir(&tmpdir, false)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e)));
        }
        try!(create_dir(&tmpdir, false));
        let volumes_base = unsafe {
            open(volbase.to_cstring().as_ptr(), O_PATH.bits())
        };
        if volumes_base < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Some(PersistentVolumeGuard {
            lock: lock,
            volumes_base: volumes_base,
            volume_name: name,
        }))
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
