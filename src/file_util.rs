use std::io;
use std::io::{Read, Write, Error};
use std::path::{Path, PathBuf};
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::os::unix::io::AsRawFd;
use std::os::unix::ffi::OsStrExt;

use nix;
use shaman::digest::Digest;
use nix::fcntl::{flock, FlockArg};

use path_util::PathExt;

pub struct Lock {
    file: fs::File,
}

pub fn read_visible_entries(dir: &Path) -> Result<Vec<PathBuf>, Error> {
    let mut res = vec!();
    for entry_ in try!(fs::read_dir(dir)) {
        let entry = try!(entry_);
        if !entry.file_name()[..].to_str().map(|x| x.starts_with("."))
            .unwrap_or(false)
        {
            res.push(entry.path().to_path_buf());
        }
    }
    Ok(res)
}

pub fn create_dir<P:AsRef<Path>>(path: P, recursive: bool) -> Result<(), Error>
{
    let path = path.as_ref();
    if path.is_dir() {
        return Ok(())
    }
    if recursive {
        match path.parent() {
            Some(p) if p != path => try!(create_dir(p, true)),
            _ => {}
        }
    }
    try!(fs::create_dir(path));
    try!(fs::set_permissions(path, fs::Permissions::from_mode(0o755)));
    Ok(())
}

pub fn create_dir_mode(path: &Path, mode: u32) -> Result<(), Error> {
    if path.is_dir() {
        return Ok(())
    }
    try!(fs::create_dir(path));
    try!(fs::set_permissions(path, fs::Permissions::from_mode(mode)));
    Ok(())
}

pub fn safe_ensure_dir(dir: &Path) -> Result<(), String> {
    match fs::symlink_metadata(dir) {
        Ok(ref stat) if stat.file_type().is_symlink() => {
            return Err(format!(concat!("The `{0}` dir can't be a symlink. ",
                               "Please run `unlink {0}`"), dir.display()));
        }
        Ok(ref stat) if stat.file_type().is_dir() => {
            // ok
        }
        Ok(_) => {
            return Err(format!(concat!("The `{0}` must be a directory. ",
                               "Please run `unlink {0}`"), dir.display()));
        }
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            try_msg!(create_dir(dir, false),
                "Can't create {dir:?}: {err}", dir=dir);
        }
        Err(ref e) => {
            return Err(format!("Can't stat `{}`: {}", dir.display(), e));
        }
    }
    return Ok(());
}

pub fn ensure_symlink(target: &Path, linkpath: &Path) -> Result<(), io::Error>
{
    match symlink(target, linkpath) {
        Ok(()) => Ok(()),
        Err(e) => {
            if e.kind() == io::ErrorKind::AlreadyExists {
                match fs::read_link(linkpath) {
                    Ok(ref path) if Path::new(path) == target => Ok(()),
                    Ok(_) => Err(e),
                    Err(e) => Err(e),
                }
            } else  {
                Err(e)
            }
        }
    }
}

pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<()>
{
    _copy(from.as_ref(), to.as_ref())
}

fn _copy(from: &Path, to: &Path) -> io::Result<()> {
    if !from.is_file() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,
                              "the source path is not an existing regular file"))
    }

    let mut reader = try!(fs::File::open(from));
    let mut writer = try!(fs::File::create(to));
    let perm = try!(reader.metadata()).permissions();

    // Smaller buffer on the stack
    // Because rust musl has very small stack (80k)
    // Allocating buffer on heap for each copy is too slow
    let mut buf = [0; 32768];
    loop {
        let len = match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(len) => len,
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        try!(writer.write_all(&buf[..len]));
    }

    try!(fs::set_permissions(to, perm));
    Ok(())
}

impl Lock {
    pub fn exclusive<P: AsRef<Path>>(p: P) -> Result<Lock, Error> {
        let f = try!(fs::File::create(p));
        try!(flock(f.as_raw_fd(), FlockArg::LockExclusiveNonblock)
            .map_err(|e| match e {
                nix::Error::Sys(code) => Error::from_raw_os_error(code as i32),
                nix::Error::InvalidPath => unreachable!(),
            }));
        Ok(Lock {
            file: f,
        })
    }
}


impl Drop for Lock {
    fn drop(&mut self) {
        flock(self.file.as_raw_fd(), FlockArg::Unlock)
            .map_err(|e| error!("Couldn't unlock file: {:?}", e)).ok();
    }
}

pub fn hash_file(path: &Path, dig: &mut Digest)
    -> Result<(), io::Error>
{
    // TODO(tailhook) include permissions and ownership into the equation
    let stat = try!(fs::symlink_metadata(path));
    if stat.file_type().is_symlink() {
        let data = try!(fs::read_link(path));
        dig.input(data.as_os_str().as_bytes());
    } else if stat.file_type().is_file() {
        let mut file = try!(fs::File::open(&path));
        loop {
            let mut chunk = [0u8; 8*1024];
            let bytes = match file.read(&mut chunk[..]) {
                Ok(0) => break,
                Ok(bytes) => bytes,
                Err(e) => return Err(e),
            };
            dig.input(&chunk[..bytes]);
        }
    }
    Ok(())
}

pub fn force_symlink(target: &Path, linkpath: &Path, perm: fs::Permissions)
    -> Result<(), io::Error>
{
    let tmpname = target.with_extension(".vagga.tmp.link~");
    try!(symlink(target, &tmpname));
    try!(fs::set_permissions(&tmpname, perm));
    try!(fs::rename(&tmpname, linkpath));
    Ok(())
}

pub fn shallow_copy(src: &Path, dest: &Path) -> Result<(), io::Error> {
    let stat = try!(fs::symlink_metadata(src));
    let typ = stat.file_type();
    if typ.is_dir() {
        try!(create_dir_mode(dest, stat.permissions().mode()));
    } else if typ.is_symlink() {
        let value = try!(fs::read_link(&src));
        try!(force_symlink(&value, dest, stat.permissions()));
    } else {
        try!(copy(src, dest));
    }
    Ok(())
}
