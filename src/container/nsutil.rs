use std::io::IoError;
use libc::c_int;
use libc::funcs::posix88::fcntl::open;
use libc::consts::os::posix88::O_RDONLY;

use super::container::{Namespace, convert_namespace};

extern {
    fn setns(fd: c_int, nstype: c_int) -> c_int;
}

pub fn set_namespace(path: &Path, ns: Namespace) -> Result<(), IoError> {
    let c_path = path.to_c_str();
    let fd = unsafe { open(c_path.as_ptr(), O_RDONLY, 0) };
    if fd < 0 {
        return Err(IoError::last_error());
    }
    let rc = unsafe { setns(fd, convert_namespace(ns)) };
    if rc < 0 {
        return Err(IoError::last_error());
    }
    return Ok(());
}
