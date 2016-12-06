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

#[cfg(not(feature="containers"))]
pub fn temporary_change_root<P, T, F, E>(path: P, mut fun: F) -> Result<T, E>
    where F: FnMut() -> Result<T, E>,
          E: From<String>,
          P: AsRef<Path>,
{
    unimplemented!();
}


#[cfg(feature="containers")]
pub fn temporary_change_root<P, T, F, E>(path: P, mut fun: F) -> Result<T, E>
    where F: FnMut() -> Result<T, E>,
          E: From<String>,
          P: AsRef<Path>,
{
    let path = path.as_ref();
    let c_root = CString::new("/").unwrap();
    if unsafe { chdir(c_root.as_ptr()) } != 0 {
        return Err(format!("Error chdir to root: {}",
                           IoError::last_os_error()).into());
    }
    if unsafe { chroot(path.to_cstring().as_ptr()) } != 0 {
        return Err(format!("Error chroot to {:?}: {}",
                           path, IoError::last_os_error()).into());
    }
    let res = fun();
    let c_pwd = CString::new(".").unwrap();
    if unsafe { chroot(c_pwd.as_ptr()) } != 0 {
        return Err(format!("Error chroot back: {}",
                           IoError::last_os_error()).into());
    }
    return res;
}

#[cfg(not(feature="containers"))]
pub fn change_root(new_root: &Path, put_old: &Path) -> Result<(), String>
{
    unimplemented!();
}

#[cfg(feature="containers")]
pub fn change_root(new_root: &Path, put_old: &Path) -> Result<(), String>
{
    let c_new_root = new_root.to_cstring();
    let c_put_old = put_old.to_cstring();
    if unsafe { pivot_root(c_new_root.as_ptr(), c_put_old.as_ptr()) } != 0 {
        return Err(format!("Error pivot_root to {:?}: {}", new_root,
                           IoError::last_os_error()));
    }
    let c_root = CString::new("/").unwrap();
    if unsafe { chdir(c_root.as_ptr()) } != 0 {
        return Err(format!("Error chdir to root: {}",
                           IoError::last_os_error()));
    }
    return Ok(());
}
