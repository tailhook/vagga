use std::ffi::CString;
use std::io::Error as IoError;
use std::path::Path;

use libc::funcs::posix88::unistd::chdir;
use libc::{c_int, c_char};


extern {
    fn chroot(dir: *const c_char) -> c_int;
    fn pivot_root(new_root: *const c_char, put_old: *const c_char) -> c_int;
}


pub fn temporary_change_root<T, F>(path: &Path, mut fun: F)
    -> Result<T, String>
    where F: FnMut() -> Result<T, String>
{
    if unsafe { chdir(CString::from_slice("/".as_bytes()).as_ptr()) } != 0 {
        return Err(format!("Error chdir to root: {}",
                           IoError::last_error()));
    }
    if unsafe { chroot(CString::from_slice(path.container_as_bytes()).as_ptr()) } != 0 {
        return Err(format!("Error chroot to {}: {}",
                           path.display(), IoError::last_error()));
    }
    let res = fun();
    if unsafe { chroot(CString::from_slice(".".container_as_bytes()).as_ptr()) } != 0 {
        return Err(format!("Error chroot back: {}",
                           IoError::last_error()));
    }
    return res;
}

pub fn change_root(new_root: &Path, put_old: &Path) -> Result<(), String>
{
    if unsafe { pivot_root(CString::from_slice(new_root.container_as_bytes()).as_ptr(),
                           CString::from_slice(put_old.container_as_bytes()).as_ptr()) } != 0 {
        return Err(format!("Error pivot_root to {}: {}", new_root.display(),
                           IoError::last_error()));
    }
    if unsafe { chdir(CString::from_slice("/".as_bytes()).as_ptr()) } != 0 {
        return Err(format!("Error chdir to root: {}",
                           IoError::last_error()));
    }
    return Ok(());
}
