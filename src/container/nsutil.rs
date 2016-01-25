use std::ffi::CString;
use std::io::Error as IoError;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use libc::{c_int};
use libc::{open, close};
use libc::O_RDONLY;
use unshare::Namespace;


static O_CLOEXEC: c_int = 0o2000000;



extern {
    fn setns(fd: c_int, nstype: c_int) -> c_int;
    fn unshare(flags: c_int) -> c_int;
}

pub fn set_namespace<P:AsRef<Path>>(path: P, ns: Namespace)
    -> Result<(), IoError>
{
    let c_path = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
    let fd = unsafe { open(c_path.as_ptr(), O_RDONLY|O_CLOEXEC, 0) };
    if fd < 0 {
        return Err(IoError::last_os_error());
    }
    let rc = unsafe { setns(fd, ns.to_clone_flag() as i32) };
    unsafe { close(fd) };
    if rc < 0 {
        return Err(IoError::last_os_error());
    }
    return Ok(());
}

pub fn unshare_namespace(ns: Namespace) -> Result<(), IoError> {
    let rc = unsafe { unshare(ns.to_clone_flag() as i32) };
    if rc < 0 {
        return Err(IoError::last_os_error());
    }
    return Ok(());
}
