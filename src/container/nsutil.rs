use std::old_io::IoError;
use std::ffi::CString;
use std::old_path::BytesContainer;
use libc::{c_int, pid_t, size_t};
use libc::funcs::posix88::fcntl::open;
use libc::funcs::posix88::unistd::close;
use libc::consts::os::posix88::O_RDONLY;

use super::container::{Namespace, convert_namespace};

static O_CLOEXEC: c_int = 0o2000000;

extern {
    fn setns(fd: c_int, nstype: c_int) -> c_int;
    fn unshare(flags: c_int) -> c_int;
    fn sethostname(name: *const u8, len: size_t) -> c_int;
}

pub fn set_namespace_fd(fd: c_int, ns: Namespace) -> Result<(), IoError> {
    let rc = unsafe { setns(fd, convert_namespace(ns)) };
    if rc < 0 {
        return Err(IoError::last_error());
    }
    return Ok(());
}

pub fn set_namespace(path: &Path, ns: Namespace) -> Result<(), IoError> {
    let c_path = CString::from_slice(path.container_as_bytes());
    let fd = unsafe { open(c_path.as_ptr(), O_RDONLY|O_CLOEXEC, 0) };
    if fd < 0 {
        return Err(IoError::last_error());
    }
    let rc = unsafe { setns(fd, convert_namespace(ns)) };
    unsafe { close(fd) };
    if rc < 0 {
        return Err(IoError::last_error());
    }
    return Ok(());
}

pub fn nsopen(pid: pid_t, ns_name: &str) -> Result<c_int, IoError> {
    let filename = CString::from_slice(
        format!("/proc/{}/ns/{}", pid, ns_name).as_bytes());
    let fd = unsafe { open(filename.as_ptr(), O_RDONLY|O_CLOEXEC, 0) };
    if fd < 0 {
        return Err(IoError::last_error());
    }
    return Ok(fd);
}

pub fn unshare_namespace(ns: Namespace) -> Result<(), IoError> {
    let rc = unsafe { unshare(convert_namespace(ns)) };
    if rc < 0 {
        return Err(IoError::last_error());
    }
    return Ok(());
}

pub fn set_hostname(name: &str) -> Result<(), IoError> {
    let rc = unsafe { sethostname(name.as_bytes().as_ptr(),
                                  name.as_bytes().len() as size_t) };
    if rc < 0 {
        return Err(IoError::last_error());
    }
    return Ok(());
}
