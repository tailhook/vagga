use std::io;
use std::fs::{read_dir, remove_dir_all, remove_file, remove_dir, copy};
use std::fs::{symlink_metadata, read_link};
use std::os::unix::fs::{symlink, MetadataExt};
use std::ptr::null;
use std::path::{Path, PathBuf};

use libc::{c_int, c_void, timeval};

use super::root::temporary_change_root;
use file_util::create_dir_mode;
use path_util::PathExt;

quick_error!{
    #[derive(Debug)]
    pub enum CopyDirError {
        ReadDir(path: PathBuf, err: io::Error) {
            display("Can't read dir {:?}: {}", path, err)
        }
        Stat(path: PathBuf, err: io::Error) {
            display("Can't stat {:?}: {}", path, err)
        }
        CopyFile(path: PathBuf, err: io::Error) {
            display("Can't copy {:?}: {}", path, err)
        }
        CreateDir(path: PathBuf, err: io::Error) {
            display("Can't create dir {:?}: {}", path, err)
        }
        ReadLink(path: PathBuf, err: io::Error) {
            display("Can't read symlink {:?}: {}", path, err)
        }
        Symlink(path: PathBuf, err: io::Error) {
            display("Can't create symlink {:?}: {}", path, err)
        }
    }
}


pub type Time = f64;

extern "C" {
    // <sys/time.h>
    fn gettimeofday(time: *mut timeval, tz: *const c_void) -> c_int;
}

pub fn clean_dir(dir: &Path, remove_dir_itself: bool) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    // We temporarily change root, so that symlinks inside the dir
    // would do no harm. But note that dir itself can be a symlink
    try!(temporary_change_root(dir, || {
        let diriter = try!(read_dir(&Path::new("/"))
             .map_err(|e| format!("Can't read directory {}: {}",
                                  dir.display(), e)));
        for entry in diriter {
            let entry = try_msg!(entry, "Error reading dir entry: {err}");
            if entry.file_type().map(|x| x.is_dir()).unwrap_or(false) {
                try_msg!(remove_dir_all(&entry.path()),
                    "Can't remove directory {dir:?}: {err}", dir=entry.path());
            } else {
                try_msg!(remove_file(&entry.path()),
                    "Can't remove file {dir:?}: {err}", dir=entry.path());
            }
        }
        Ok(())
    }));
    if remove_dir_itself {
        try_msg!(remove_dir(dir),
            "Can't remove dir {dir:?}: {err}", dir=dir);
    }
    return Ok(());
}

pub fn get_time() -> Time {
    let mut tv = timeval { tv_sec: 0, tv_usec: 0 };
    unsafe { gettimeofday(&mut tv, null()) };
    return tv.tv_sec as f64 + 0.000001 * tv.tv_usec as f64;
}

pub fn copy_dir(old: &Path, new: &Path) -> Result<(), CopyDirError> {
    use self::CopyDirError::*;
    // TODO(tailhook) use reflinks if supported
    for item in try!(read_dir(old).map_err(|e| ReadDir(old.to_path_buf(), e))) {
        let entry = try!(item.map_err(|e| ReadDir(old.to_path_buf(), e)));
        let nitem = new.join(entry.file_name());
        let epath = entry.path();
        let typ = try!(entry.file_type().map_err(|e| Stat(epath.clone(), e)));
        if typ.is_file() {
            try!(copy(&epath, &nitem) .map_err(|e| CopyFile(epath.clone(), e)));
        } else if typ.is_dir() {
            let stat = try!(
                symlink_metadata(&epath).map_err(|e| Stat(epath.clone(), e)));
            if !nitem.is_dir() {
                try!(create_dir_mode(&nitem, stat.mode())
                    .map_err(|e| CreateDir(epath.clone(), e)));
            }
            try!(copy_dir(&epath, &nitem));
        } else if typ.is_symlink() {
            let lnk = try!(read_link(&epath)
                           .map_err(|e| ReadLink(epath.clone(), e)));
            try!(symlink(&lnk, &nitem).map_err(|e| Symlink(epath.clone(), e)));
        } else {
            warn!("Unknown file type {:?}", &entry.path());
        }
    }
    Ok(())
}
