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
        CopyFile(src: PathBuf, dst: PathBuf, err: io::Error) {
            display("Can't copy {:?} -> {:?}: {}", src, dst, err)
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
    let dir = try!(read_dir(old).map_err(|e| ReadDir(old.to_path_buf(), e)));
    let mut stack = vec![dir];
    let mut oldp = old.to_path_buf();
    let mut newp = new.to_path_buf();
    'next_dir: while let Some(mut dir) = stack.pop() {
        while let Some(item) = dir.next() {
            let entry = try!(item.map_err(|e| ReadDir(old.to_path_buf(), e)));
            let filename = entry.file_name();
            oldp.push(&filename);
            newp.push(&filename);

            let typ = try!(entry.file_type()
                .map_err(|e| Stat(oldp.clone(), e)));
            if typ.is_file() {
                try!(copy(&oldp, &newp)
                    .map_err(|e| CopyFile(oldp.clone(), newp.clone(), e)));
            } else if typ.is_dir() {
                let stat = try!(symlink_metadata(&oldp)
                    .map_err(|e| Stat(oldp.clone(), e)));
                if !newp.is_dir() {
                    try!(create_dir_mode(&newp, stat.mode())
                        .map_err(|e| CreateDir(newp.clone(), e)));
                }
                stack.push(dir);  // Return dir to stack
                let ndir = try!(read_dir(&oldp)
                    .map_err(|e| ReadDir(oldp.to_path_buf(), e)));
                stack.push(ndir); // Add new dir to the stack too
                continue 'next_dir;
            } else if typ.is_symlink() {
                let lnk = try!(read_link(&oldp)
                               .map_err(|e| ReadLink(oldp.clone(), e)));
                try!(symlink(&lnk, &newp)
                    .map_err(|e| Symlink(newp.clone(), e)));
            } else {
                warn!("Unknown file type {:?}", &entry.path());
            }
            oldp.pop();
            newp.pop();
        }
        oldp.pop();
        newp.pop();
    }
    Ok(())
}
