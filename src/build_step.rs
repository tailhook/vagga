use std::fs;
use std::io::{self, Read};
use std::rc::Rc;
use std::fmt::{Debug, Display};
use std::path::Path;
use std::os::unix::raw::{uid_t, gid_t};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{PermissionsExt, MetadataExt};

use shaman::sha2;
use shaman::digest::Digest as DigestTrait;

// Convenient reexports
pub use version::{Error as VersionError};
pub use builder::{StepError, Guard};
pub use config::Config;

#[derive(Clone, Debug)]
pub struct Step(pub Rc<BuildStep>);

pub struct Digest(sha2::Sha256);

pub trait BuildStep: Debug {
    fn hash(&self, cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>;
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>;
    fn is_dependent_on(&self) -> Option<&str>;
}

impl BuildStep for Step {
    fn hash(&self, cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        self.0.hash(cfg, hash)
    }
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        self.0.build(guard, build)
    }
    fn is_dependent_on(&self) -> Option<&str>
    {
        self.is_dependent_on()
    }
}

impl Digest {
    pub fn new() -> Digest {
        Digest(sha2::Sha256::new())
    }
    // TODO(tailhook) get rid of the method
    pub fn unwrap(self) -> sha2::Sha256 {
        return self.0
    }
    pub fn item<V: AsRef<[u8]>>(&mut self, value: V) {
        self.0.input(value.as_ref());
        self.0.input(b"\0");
    }
    pub fn field<K: AsRef<[u8]>, V: AsRef<[u8]>>(&mut self, key: K, value: V) {
        self.0.input(key.as_ref());
        self.0.input(b"\0");
        self.0.input(value.as_ref());
        self.0.input(b"\0");
    }
    pub fn text<K: AsRef<[u8]>, V: Display>(&mut self, key: K, value: V) {
        self.0.input(key.as_ref());
        self.0.input(b"\0");
        self.0.input(format!("{}", value).as_bytes());
        self.0.input(b"\0");
    }
    pub fn opt_field<K: AsRef<[u8]>, V: AsRef<[u8]>>(&mut self,
        key: K, value: &Option<V>)
    {
        if let Some(ref val) = *value {
            self.0.input(key.as_ref());
            self.0.input(b"\0");
            self.0.input(val.as_ref());
            self.0.input(b"\0");
        }
    }
    pub fn bool<K: AsRef<[u8]>>(&mut self, key: K, value: bool)
    {
        self.0.input(key.as_ref());
        self.0.input(b"\0");
        self.0.input(if value { b"0" } else { b"1" });
    }
    pub fn sequence<K, T>(&mut self, key: K, seq: &[T])
        where K: AsRef<[u8]>, T: AsRef<[u8]>
    {
        self.0.input(key.as_ref());
        self.0.input(b"\0");
        for value in seq {
            self.0.input(value.as_ref());
            self.0.input(b"\0");
        }
    }
    pub fn file(&mut self, path: &Path,
        owner_uid: Option<uid_t>, owner_gid: Option<gid_t>)
        -> Result<(), io::Error>
    {
        // TODO(tailhook) include permissions and ownership into the equation
        let stat = try!(fs::symlink_metadata(path));
        self.field("filename", path.as_os_str().as_bytes());
        self.text("mode", stat.permissions().mode());
        self.text("uid", owner_uid.unwrap_or(stat.uid()));
        self.text("gid", owner_gid.unwrap_or(stat.gid()));
        if stat.file_type().is_symlink() {
            let data = try!(fs::read_link(path));
            self.0.input(data.as_os_str().as_bytes());
        } else if stat.file_type().is_file() {
            let mut file = try!(fs::File::open(&path));
            loop {
                let mut chunk = [0u8; 8*1024];
                let bytes = match file.read(&mut chunk[..]) {
                    Ok(0) => break,
                    Ok(bytes) => bytes,
                    Err(e) => return Err(e),
                };
                self.0.input(&chunk[..bytes]);
            }
        }
        Ok(())
    }
}
