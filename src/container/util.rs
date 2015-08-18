use std::ffi::CStr;
use std::fs::{read_dir, remove_dir_all, remove_file, remove_dir, copy};
use std::fs::{symlink_metadata, read_link};
use std::os::unix::fs::{symlink, MetadataExt};
use std::ptr::null;
use std::path::Path;

use libc::{c_int, uid_t, gid_t, c_char, c_void, timeval};

use super::root::temporary_change_root;
use file_util::create_dir_mode;
use path_util::PathExt;


pub type Time = f64;

// pwd.h
#[repr(C)]
struct passwd {
    pw_name: *mut c_char,       /* username */
    pw_passwd: *mut u8,     /* user password */
    pw_uid: uid_t,      /* user ID */
    pw_gid: gid_t,      /* group ID */
    pw_gecos: *mut u8,      /* user information */
    pw_dir: *mut u8,        /* home directory */
    pw_shell: *mut u8,      /* shell program */
}

extern "C" {
    // pwd.h
    fn getpwuid(uid: uid_t) -> *const passwd;
    // <sys/time.h>
    fn gettimeofday(time: *mut timeval, tz: *const c_void) -> c_int;
}

pub fn get_user_name(uid: uid_t) -> Result<String, String> {
    unsafe {
        let val = getpwuid(uid);
        if val != null() {
            return String::from_utf8(
                CStr::from_ptr((*val).pw_name).to_bytes().to_vec())
                .map_err(|e| format!("Can't decode user name: {}", e));
        }
    }
    return Err(format!("User {} not found", uid));
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

pub fn copy_dir(old: &Path, new: &Path) -> Result<(), String> {
    // TODO(tailhook) use reflinks if supported
    for item in try_msg!(read_dir(old), "Can't open dir {d:?}: {err}", d=old) {
        let entry = try_msg!(item, "Can't read dir entry {d:?}: {err}", d=old);
        let nitem = new.join(entry.file_name());
        let typ = try_msg!(entry.file_type(),
            "Can't stat {f:?}: {err}", f=entry.path());
        if typ.is_file() {
            try!(copy(entry.path(), &nitem)
                .map_err(|e| format!("Can't hard-link file: {}", e)));
        } else if typ.is_dir() {
            let stat = try_msg!(symlink_metadata(entry.path()),
                "Can't stat file {path:?}: {err}", path=entry.path());
            if !nitem.is_dir() {
                try_msg!(create_dir_mode(&nitem, stat.mode()),
                    "Can't create dir {dir:?}: {err}", dir=nitem);
            }
            try!(copy_dir(&entry.path(), &nitem));
        } else if typ.is_symlink() {
            let lnk = try!(read_link(entry.path())
                .map_err(|e| format!("Can't readlink: {}", e)));
            try!(symlink(&lnk, &nitem)
                .map_err(|e| format!("Can't symlink: {}", e)));
        } else {
            warn!("Unknown file type {:?}", entry.path());
        }
    }
    Ok(())
}
