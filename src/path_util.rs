use std::env;
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};


// TODO(tailhook) probably get rid of this after migrating to unshare crate
pub trait ToCString {
    fn to_cstring(&self) -> CString;
}

impl ToCString for Path {
    fn to_cstring(&self) -> CString {
        CString::new(self.as_os_str().as_bytes()).unwrap()
    }
}

impl<'a, T:AsRef<[u8]>> ToCString for T {
    fn to_cstring(&self) -> CString {
        CString::new(self.as_ref()).unwrap()
    }
}

pub trait Expand {
    fn expand_home(self) -> Result<PathBuf, ()>;
}

impl Expand for PathBuf {
    fn expand_home(self) -> Result<PathBuf, ()> {
        if !self.starts_with("~") {
            return Ok(self);
        }
        let mut it = self.iter();
        it.next();
        if let Some(home) = env::var_os("_VAGGA_HOME") {
            let home = Path::new(&home);
            Ok(home.join(it.as_path()))
        } else {
            Err(())
        }
    }
}
