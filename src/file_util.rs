use std::io;
use std::io::{Read, Write, Error};
use std::path::{Path, PathBuf};
use std::fs;
use std::ffi::CString;
use std::os::unix::fs::{PermissionsExt, MetadataExt, symlink};
use std::os::unix::io::AsRawFd;
use std::os::unix::ffi::OsStrExt;

use nix;
use libc::{uid_t, gid_t, c_int, utime, utimbuf};
use nix::fcntl::{flock, FlockArg};

extern "C" {
    fn lchown(path: *const i8, owner: uid_t, group: gid_t) -> c_int;
}

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

    try!(copy_stream(&mut reader, &mut writer));

    try!(fs::set_permissions(to, perm));
    Ok(())
}

pub fn copy_stream(reader: &mut Read, writer: &mut Write)
    -> io::Result<()>
{
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
    pub fn exclusive_wait<P: AsRef<Path>>(path: P, message: &str)
        -> Result<Lock, Error>
    {
        let f = try!(fs::File::create(path));
        match flock(f.as_raw_fd(), FlockArg::LockExclusiveNonblock) {
            Ok(()) => {}
            Err(nix::Error::Sys(nix::Errno::EAGAIN)) => {
                warn!("{}", message);
                try!(flock(f.as_raw_fd(), FlockArg::LockExclusive)
                    .map_err(|e| match e {
                        nix::Error::Sys(code) => {
                            Error::from_raw_os_error(code as i32)
                        },
                        nix::Error::InvalidPath => unreachable!(),
                    }));
            }
            Err(nix::Error::Sys(code)) => {
                return Err(Error::from_raw_os_error(code as i32))
            }
            Err(nix::Error::InvalidPath) => unreachable!(),
        }
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

pub fn check_stream_hashsum(mut reader: &mut Read, sha256: &String) -> Result<(), String>
{
    use digest::Digest;
    use sha2::Digest as Sha2Digest;

    let mut hash = Digest::new();
    try_msg!(hash.stream(&mut reader),
        "Error when calculating hashsum: {err}");
    let hash_str = hash.unwrap().result_str();
    if !hash_str.starts_with(sha256) {
        return Err(format!("Hashsum mismatch: expected {} but was {}",
            sha256, hash_str));
    }
    Ok(())
}

pub fn force_symlink(target: &Path, linkpath: &Path)
    -> Result<(), io::Error>
{
    let tmpname = linkpath.with_extension(".vagga.tmp.link~");
    try!(symlink(target, &tmpname));
    try!(fs::rename(&tmpname, linkpath));
    Ok(())
}

pub fn set_owner_group(target: &Path, uid: uid_t, gid: gid_t)
    -> Result<(), io::Error>
{
    let rc = unsafe { lchown(
        CString::new(target.as_os_str().as_bytes()).unwrap().as_ptr(),
        uid, gid) };
    if rc != 0 {
        warn!("Can't chown {:?}: {}", target, io::Error::last_os_error());
        Ok(())
        // Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

/// Shallow copy of file/dir/symlink
///
/// The owner_uid/owner_gid parameters optionally override the owner
///
/// Returns false if path is a directory
pub fn shallow_copy(src: &Path, dest: &Path,
    owner_uid: Option<uid_t>, owner_gid: Option<gid_t>)
    -> Result<bool, io::Error>
{
    let stat = try!(fs::symlink_metadata(src));
    let typ = stat.file_type();
    let mut is_dir = false;
    if typ.is_dir() {
        is_dir = true;
        let nstat = fs::symlink_metadata(dest);
        // We don't change permissions and owner of already created directories
        //
        // There are couple of reasons:
        // 1. We consider such overlaps a "merge" of a directory, so it's
        //    unclear which one should override the permissions
        // 2. Changing permissions of the root directory of the copied system
        //    is usually counter-intuitive (so we contradict rsync)
        // 3. Some directories (/proc, /sys, /dev) are mount points and we
        //    can't change the permissions
        if nstat.is_err() {
            try!(fs::create_dir(dest));
            try!(fs::set_permissions(dest, stat.permissions()));
            try!(set_owner_group(dest,
                owner_uid.unwrap_or(stat.uid()),
                owner_gid.unwrap_or(stat.gid())));
        }
    } else if typ.is_symlink() {
        let value = try!(fs::read_link(&src));
        try!(force_symlink(&value, dest));
        try!(set_owner_group(dest,
            owner_uid.unwrap_or(stat.uid()),
            owner_gid.unwrap_or(stat.gid())));
    } else {
        try!(copy(src, dest));
        try!(set_owner_group(dest,
            owner_uid.unwrap_or(stat.uid()),
            owner_gid.unwrap_or(stat.gid())));
    }
    Ok(!is_dir)
}

pub fn copy_utime<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q)
    -> io::Result<()>
{
    let metadata = try!(fs::metadata(from.as_ref()));
    let filename = CString::new(to.as_ref().as_os_str().as_bytes())
                   .unwrap().as_ptr();
    let utimes = utimbuf {
        actime: metadata.atime(),
        modtime: metadata.mtime(),
    };
    let rc = unsafe { utime(filename, &utimes) };
    if rc != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}
