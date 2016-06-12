use std::io::{self, Write, Read};
use std::fs;
use std::fmt::Display;
use std::path::Path;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{PermissionsExt, MetadataExt};

use libc::{uid_t, gid_t};
use sha2::Sha256;
use sha2::Digest as DigestTrait;

pub struct Digest(Sha256);


impl Digest {
    pub fn new() -> Digest {
        Digest(Sha256::new())
    }
    // TODO(tailhook) get rid of the method
    pub fn unwrap(self) -> Sha256 {
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
            try!(self.stream(&mut file));
        }
        Ok(())
    }
    pub fn stream(&mut self, reader: &mut Read)
        -> Result<(), io::Error>
    {
        let mut buf = [0u8; 8*1024];
        loop {
            let len = match reader.read(&mut buf[..]) {
                Ok(0) => break,
                Ok(len) => len,
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            };
            self.0.input(&buf[..len]);
        }
        Ok(())
    }
}

impl Write for Digest {
    fn write(&mut self, chunk: &[u8]) -> io::Result<usize> {
        self.0.input(chunk);
        Ok(chunk.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
