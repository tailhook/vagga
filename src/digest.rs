use std::io::{self, Write, Read};
use std::fmt::Display;

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
    pub fn input<V: AsRef<[u8]>>(&mut self, value: V) {
        self.0.input(value.as_ref());
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
    pub fn sequence<K, I: IntoIterator>(&mut self, key: K, seq: I)
        where K: AsRef<[u8]>, I::Item: AsRef<[u8]>
    {
        self.0.input(key.as_ref());
        self.0.input(b"\0");
        for value in seq {
            self.0.input(value.as_ref());
            self.0.input(b"\0");
        }
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
