use std::env;
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::path::Component::RootDir;
use std::fs::metadata;


pub trait ToRelative {
    fn rel<'x>(&'x self) -> &'x Path;
    fn rel_to<'x>(&'x self, &'x Path) -> Option<&'x Path>;
    fn is_ancestor(&self, &Path) -> bool;
}

impl ToRelative for Path {
    fn rel<'x>(&'x self) -> &'x Path {
        let mut iter = self.components();
        assert!(iter.next() == Some(RootDir));
        iter.as_path()
    }
    fn rel_to<'x>(&'x self, other: &'x Path) -> Option<&'x Path> {
        let mut iter = self.components();
        for (their, my) in other.components().zip(iter.by_ref()) {
            if my != their {
                return None;
            }
        }
        Some(iter.as_path())
    }
    fn is_ancestor(&self, path: &Path) -> bool {
      return self.rel_to(path).is_some();
    }
}

impl ToRelative for PathBuf {
    fn rel<'x>(&'x self) -> &'x Path {
        self.as_path().rel()
    }
    fn rel_to<'x>(&'x self, other: &'x Path) -> Option<&'x Path> {
        self.as_path().rel_to(other)
    }
    fn is_ancestor(&self, path: &Path) -> bool {
      return self.rel_to(path).is_some();
    }
}

// Should resemble std::fs::PathExt inteface;
pub trait PathExt {
    fn exists(&self) -> bool;
    fn is_dir(&self) -> bool;
}

impl PathExt for Path {
    fn exists(&self) -> bool {
        metadata(self).is_ok()
    }
    fn is_dir(&self) -> bool {
        metadata(self).map(|x| x.is_dir()).unwrap_or(false)
    }
}

impl PathExt for PathBuf {
    fn exists(&self) -> bool {
        self.as_path().exists()
    }
    fn is_dir(&self) -> bool {
        self.as_path().is_dir()
    }
}

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
        if let Some(home) = env::var_os("VAGGA_USER_HOME") {
            let home = Path::new(&home);
            Ok(home.join(it.as_path()))
        } else {
            Err(())
        }
    }
}
