use std::str;
use std::io::{self, Write, Read};
use std::fmt::{self, Write as WriteFmt};
use std::path::{Path, PathBuf};
use std::os::unix::ffi::OsStrExt;

use sha2::{Sha256, Digest as DigestTrait};
use digest_writer::Writer;
use rustc_serialize::json::Json;
use config::Range;

/// This is a wrapper that has convenience methods for hashing in vagga
/// commands
pub struct Digest {
    sha: DebugWriter,
    debug: Opt<String>,
}

/// This is internal trait only
pub trait Digestable {
    fn digest(&self, title: &str, dig: &mut Digest);
}

enum Opt<W> {
    Out(W),
    Sink,
}

/// This just copies the hash data into a buffer for debugging
struct DebugWriter {
    sha: Writer<Sha256>,
    data: Opt<Vec<u8>>,
}

/// A wrapper type used for hexlification, use `hex()` function
pub struct ShowHex<'a>(&'a Sha256);

static LOWER_CHARS: &'static[u8] = b"0123456789abcdef";


impl Digest {
    pub fn new(debug: bool, raw_debug: bool) -> Digest {
        Digest {
            sha: DebugWriter {
                sha: Writer::new(Sha256::new()),
                data: if raw_debug { Opt::Out(Vec::new()) } else { Opt::Sink },
            },
            debug: if debug { Opt::Out(String::new()) } else { Opt::Sink },
        }
    }
    //
    // --- adding something to digests
    //
    pub fn field<D: Digestable>(&mut self, key: &str, value: D) {
        value.digest(key, self);
    }
    pub fn command(&mut self, name: &str) {
        write!(&mut self.sha, "COMMAND\0{}\0", name).unwrap();
        write!(&mut self.debug, "----- Command {} -----\n", name).unwrap();
    }
    /// This only outputs if field is not None
    ///
    /// This method may be used for adding fields which are None by default,
    /// while maintaining backwards compatibility
    pub fn opt_field<D: Digestable>(&mut self, key: &str, value: &Option<D>) {
        if let Some(ref val) = *value {
            self.field(key, val);
        }
    }
    pub fn file(&mut self, name: &Path, reader: &mut Read)
        -> Result<(), io::Error>
    {
        io::copy(reader, &mut self.sha)?;
        write!(&mut self.debug, "file {:?}\n", name).unwrap();
        Ok(())
    }
    //
    // --- End of digest-adding methods
    //

    pub fn print_debug_info(&self) {
        match self.debug {
            Opt::Out(ref x) => println!("{}", x),
            Opt::Sink => {}, // unreachable?
        }
    }

    pub fn dump_info(&self) {
        match self.sha.data {
            Opt::Out(ref x) => io::stdout().write_all(x).unwrap(),
            Opt::Sink => {}, // unreachable?
        }
    }
}

impl<W: io::Write> io::Write for Opt<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        use self::Opt::*;
        match *self {
            Out(ref mut x) => x.write(buf),
            Sink => Ok(buf.len())
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        use self::Opt::*;
        match *self {
            Out(ref mut x) => x.flush(),
            Sink => Ok(())
        }
    }
}

impl io::Write for DebugWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.data.write(buf)?;
        self.sha.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.data.flush()?;
        self.sha.flush()
    }
}

impl<W: fmt::Write> fmt::Write for Opt<W> {
    fn write_str(&mut self, str: &str) -> fmt::Result {
        use self::Opt::*;
        match *self {
            Out(ref mut x) => x.write_str(str),
            Sink => Ok(())
        }
    }
}

impl fmt::LowerHex for Digest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        hexfmt(&self.sha.sha.clone().result()[..], f)
    }
}

fn hexfmt(data: &[u8], f: &mut fmt::Formatter) -> fmt::Result {
    assert!(data.len() == 32);
    let max_digits = f.precision().unwrap_or(data.len());
    let mut res = [0u8; 64];
    for (i, c) in data.iter().take(max_digits).enumerate() {
        res[i*2] = LOWER_CHARS[(c >> 4) as usize];
        res[i*2+1] = LOWER_CHARS[(c & 0xF) as usize];
    }
    f.write_str(unsafe {
        str::from_utf8_unchecked(&res[..max_digits*2])
    })?;
    Ok(())
}

impl<'a> fmt::LowerHex for ShowHex<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        hexfmt(&self.0.result()[..], f)
    }
}

impl Digestable for String {
    fn digest(&self, title: &str, dig: &mut Digest) {
        display_field(self, title, dig)
    }
}

impl Digestable for bool {
    fn digest(&self, title: &str, dig: &mut Digest) {
        display_field(self, title, dig)
    }
}

impl<'a> Digestable for &'a str {
    fn digest(&self, title: &str, dig: &mut Digest) {
        display_field(self, title, dig)
    }
}

impl Digestable for u32 {
    fn digest(&self, title: &str, dig: &mut Digest) {
        display_field(self, title, dig)
    }
}

fn display_field<T: fmt::Display>(value: T, title: &str, dig: &mut Digest) {
    write!(&mut dig.sha, "{}\0{}\0", title, value).unwrap();
    write!(&mut dig.debug, "field {:?} {}\n", title, value).unwrap();
}

fn path_field<T: AsRef<Path>>(value: T, title: &str, dig: &mut Digest) {
    write!(&mut dig.sha, "{}\0", title).unwrap();
    dig.sha.write_all(value.as_ref().as_os_str().as_bytes()).unwrap();
    dig.sha.write_all(&[0]).unwrap();
    write!(&mut dig.debug,
        "field:path {:?} {:?}\n", title, value.as_ref()).unwrap();
}

impl Digestable for Json {
    fn digest(&self, title: &str, dig: &mut Digest) {
        write!(&mut dig.sha, "{}\0{}\0", title, self).unwrap();
        write!(&mut dig.debug, "field:json {:?} {}\n", title, self).unwrap();
    }
}

impl<'a> Digestable for &'a Path {
    fn digest(&self, title: &str, dig: &mut Digest) {
        path_field(self, title, dig)
    }
}

impl<> Digestable for PathBuf {
    fn digest(&self, title: &str, dig: &mut Digest) {
        path_field(self, title, dig)
    }
}

impl<'a> Digestable for &'a Vec<String> {
    fn digest(&self, title: &str, dig: &mut Digest) {
        write!(&mut dig.sha, "{}\0", title).unwrap();
        for val in *self {
            write!(&mut dig.sha, "{}\0", val).unwrap();
        }
        write!(&mut dig.debug, "field:list {:?} {:?}\n", title, self).unwrap();
    }
}

impl<'a> Digestable for &'a Vec<PathBuf> {
    fn digest(&self, title: &str, dig: &mut Digest) {
        write!(&mut dig.sha, "{}\0", title).unwrap();
        for val in *self {
            dig.sha.write_all(val.as_os_str().as_bytes()).unwrap();
            dig.sha.write_all(&[0]).unwrap();
        }
        write!(&mut dig.debug, "field:list {:?} {:?}\n", title, self).unwrap();
    }
}

impl<'a> Digestable for &'a Vec<Range> {
    fn digest(&self, title: &str, dig: &mut Digest) {
        write!(&mut dig.sha, "{}\0", title).unwrap();
        for val in *self {
            write!(&mut dig.sha, "{}-{}\0", val.start(), val.end()).unwrap();
        }
        write!(&mut dig.debug, "field:list {:?} {:?}\n", title, self).unwrap();
    }
}

impl<'a> Digestable for &'a Vec<u32> {
    fn digest(&self, title: &str, dig: &mut Digest) {
        write!(&mut dig.sha, "{}\0", title).unwrap();
        for val in *self {
            write!(&mut dig.sha, "{}\0", val).unwrap();
        }
        write!(&mut dig.debug, "field:list {:?} {:?}\n", title, self).unwrap();
    }
}

impl<'a, T: Digestable> Digestable for &'a T {
    fn digest(&self, title: &str, dig: &mut Digest) {
        (*self).digest(title, dig)
    }
}


/// Zero-copy formatting of hash value with or without precision
pub fn hex(src: &Sha256) -> ShowHex {
    ShowHex(&src)
}
