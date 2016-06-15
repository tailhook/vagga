use std::ffi::CString;
use std::io::Error as IoError;
use std::path::Path;

use libc::chdir;
use libc::{c_int, c_char};

use path_util::ToCString;


extern {
    fn chroot(dir: *const c_char) -> c_int;
    fn pivot_root(new_root: *const c_char, put_old: *const c_char) -> c_int;
}


pub fn temporary_change_root<P, T, F, E>(path: P, mut fun: F) -> Result<T, E>
    where F: FnMut() -> Result<T, E>,
          E: From<String>,
          P: AsRef<Path>,
{
    let path = path.as_ref();
    if unsafe { chdir(CString::new("/").unwrap().as_ptr()) } != 0 {
        return Err(format!("Error chdir to root: {}",
                           IoError::last_os_error()).into());
    }
    if unsafe { chroot(path.to_cstring().as_ptr()) } != 0 {
        return Err(format!("Error chroot to {:?}: {}",
                           path, IoError::last_os_error()).into());
    }
    let res = fun();
    if unsafe { chroot(CString::new(".").unwrap().as_ptr()) } != 0 {
        return Err(format!("Error chroot back: {}",
                           IoError::last_os_error()).into());
    }
    return res;
}

pub fn change_root(new_root: &Path, put_old: &Path) -> Result<(), String>
{
    if unsafe { pivot_root(CString::new(new_root.to_str().unwrap()).unwrap().as_ptr(),
                           CString::new(put_old.to_str().unwrap()).unwrap().as_ptr()) } != 0 {
        return Err(format!("Error pivot_root to {:?}: {}", new_root,
                           IoError::last_os_error()));
    }
    if unsafe { chdir(CString::new("/").unwrap().as_ptr()) } != 0 {
        return Err(format!("Error chdir to root: {}",
                           IoError::last_os_error()));
    }
    return Ok(());
}
