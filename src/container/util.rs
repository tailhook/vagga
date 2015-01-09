use std::ptr::null;
use std::string::raw::from_buf;
use std::io::fs::{readdir, rmdir_recursive, unlink, rmdir};
use std::io::fs::PathExtensions;
use libc::{uid_t, gid_t, c_char};

use super::root::temporary_change_root;

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
}

pub fn get_user_name(uid: uid_t) -> Result<String, String> {
    unsafe {
        let val = getpwuid(uid);
        if val != null() {
            return Ok(from_buf((*val).pw_name as *const u8));
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
        let dirlist = try!(readdir(&Path::new("/"))
             .map_err(|e| format!("Can't read directory {}: {}",
                                  dir.display(), e)));
        for path in dirlist.into_iter() {
            if path.is_dir() {
                try!(rmdir_recursive(&path)
                    .map_err(|e| format!("Can't remove directory {}{}: {}",
                        dir.display(), path.display(), e)));
            } else {
                try!(unlink(&path)
                    .map_err(|e| format!("Can't remove file {}{}: {}",
                        dir.display(), path.display(), e)));
            }
        }
        Ok(())
    }));
    if remove_dir_itself {
        try!(rmdir(dir).map_err(|e| format!("Can't remove dir {}: {}",
                                            dir.display(), e)));
    }
    return Ok(());
}
