use std::io;
use std::io::{Read, Write, Error};
use std::path::{Path, PathBuf};
use std::fs;
use std::fs::Metadata;
use std::ffi::CString;
use std::time::{Instant, Duration};
use std::os::unix::fs::{PermissionsExt, MetadataExt, symlink};
use std::os::unix::io::AsRawFd;
use std::os::unix::ffi::OsStrExt;

use nix;
use libc::{uid_t, gid_t, c_int, utime, utimbuf};
use nix::fcntl::{flock, FlockArg};

use path_util::ToCString;

extern "C" {
    fn lchown(path: *const i8, owner: uid_t, group: gid_t) -> c_int;
}

pub struct Lock {
    file: fs::File,
}

pub fn read_visible_entries(dir: &Path) -> Result<Vec<PathBuf>, Error> {
    let mut res = vec!();
    for entry_ in fs::read_dir(dir)? {
        let entry = entry_?;
        if !entry.file_name()[..].to_str().map(|x| x.starts_with("."))
            .unwrap_or(false)
        {
            res.push(entry.path().to_path_buf());
        }
    }
    Ok(res)
}

pub struct Dir<'a> {
    path: &'a Path,
    recursive: bool,
    mode: Option<u32>,
    uid: Option<uid_t>,
    gid: Option<gid_t>,
}

impl<'a> Dir<'a> {
    pub fn new<P: ?Sized>(path: &'a P) -> Dir<'a>
        where P: AsRef<Path>
    {
        Dir {
            path: path.as_ref(),
            recursive: false,
            mode: None,
            uid: None,
            gid: None,
        }
    }

    pub fn recursive(&mut self, recursive: bool) -> &mut Dir<'a> {
        self.recursive = recursive;
        self
    }

    pub fn mode(&mut self, mode: u32) -> &mut Dir<'a> {
        self.mode = Some(mode);
        self
    }

    pub fn uid(&mut self, uid: uid_t) -> &mut Dir<'a> {
        self.uid = Some(uid);
        self
    }

    pub fn gid(&mut self, gid: gid_t) -> &mut Dir<'a> {
        self.gid = Some(gid);
        self
    }

    pub fn create(&self) -> Result<(), Error> {
        self._create(self.path, true)
    }

    fn _create(&self, path: &Path, is_last: bool)
        -> Result<(), Error>
    {
        if path.is_dir() {
            return Ok(())
        }
        if self.recursive {
            match path.parent() {
                Some(p) if p != path => {
                    self._create(p, false)?;
                }
                _ => {}
            }
        }
        fs::create_dir(path)?;
        let mode = if is_last { self.mode } else { None };
        fs::set_permissions(path,
            fs::Permissions::from_mode(mode.unwrap_or(0o755)))?;
        if is_last {
            if self.uid.is_some() || self.gid.is_some() {
                let uid = if let Some(uid) = self.uid {
                    uid
                } else {
                    path.symlink_metadata()?.uid()
                };
                let gid = if let Some(gid) = self.gid {
                    gid
                } else {
                    path.symlink_metadata()?.gid()
                };
                set_owner_group(path, uid, gid)?;
            }
        }
        Ok(())
    }
}

pub fn safe_ensure_dir(dir: &Path) -> Result<(), String> {
    match fs::symlink_metadata(dir) {
        Ok(ref stat) if stat.file_type().is_symlink() => {
            return Err(format!(concat!("The `{0:?}` dir can't be a symlink. ",
                               "Please run `unlink {0:?}`"), dir));
        }
        Ok(ref stat) if stat.file_type().is_dir() => {
            // ok
        }
        Ok(_) => {
            return Err(format!(concat!("The {0:?} must be a directory. ",
                               "Please run `unlink {0:?}`"), dir));
        }
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            try_msg!(Dir::new(dir).create(),
                "Can't create {dir:?}: {err}", dir=dir);
        }
        Err(ref e) => {
            return Err(format!("Can't stat {:?}: {}", dir, e));
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
    _copy(from.as_ref(), to.as_ref(), None)
}

pub fn copy_with_mode<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q, mode: u32)
    -> io::Result<()>
{
    _copy(from.as_ref(), to.as_ref(), Some(mode))
}

fn _copy(from: &Path, to: &Path, mode: Option<u32>) -> io::Result<()> {
    if !from.is_file() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,
                              "the source path is not an existing regular file"))
    }

    let mut reader = fs::File::open(from)?;
    let mut writer = fs::File::create(to)?;

    copy_stream(&mut reader, &mut writer)?;

    let perm = match mode {
        Some(mode) => {
            fs::Permissions::from_mode(mode)
        },
        None => {
            reader.metadata()?.permissions()
        }
    };
    fs::set_permissions(to, perm)?;
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
        writer.write_all(&buf[..len])?;
    }
    Ok(())
}

impl Lock {
    pub fn exclusive<P: AsRef<Path>>(p: P) -> Result<Lock, Error> {
        let f = fs::File::create(p)?;
        flock(f.as_raw_fd(), FlockArg::LockExclusiveNonblock)
            .map_err(|e| match e {
                nix::Error::Sys(code) => Error::from_raw_os_error(code as i32),
                nix::Error::InvalidPath => unreachable!(),
            })?;
        Ok(Lock {
            file: f,
        })
    }
    pub fn exclusive_wait<P: AsRef<Path>>(path: P, message: &str)
        -> Result<Lock, Error>
    {
        let f = fs::File::create(path)?;
        match flock(f.as_raw_fd(), FlockArg::LockExclusiveNonblock) {
            Ok(()) => {}
            Err(nix::Error::Sys(nix::Errno::EAGAIN)) => {
                warn!("{}", message);
                let lock_start = Instant::now();
                flock(f.as_raw_fd(), FlockArg::LockExclusive)
                    .map_err(|e| match e {
                        nix::Error::Sys(code) => {
                            Error::from_raw_os_error(code as i32)
                        },
                        nix::Error::InvalidPath => unreachable!(),
                    })?;
                let elapsed = lock_start.elapsed();
                if elapsed > Duration::new(5, 0) {
                    warn!("Lock was held {} seconds. Proceeding...",
                        elapsed.as_secs());
                }
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

pub fn check_stream_hashsum(mut reader: &mut Read, sha256: &String)
    -> Result<(), String>
{
    use sha2::Sha256;
    use digest_writer::Writer;
    use digest::hex;

    let mut hash = Writer::new(Sha256::new());
    try_msg!(io::copy(&mut reader, &mut hash),
        "Error when calculating hashsum: {err}");
    let hash_str = format!("{:x}", hex(&hash.into_inner()));
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
    symlink(target, &tmpname)?;
    fs::rename(&tmpname, linkpath)?;
    Ok(())
}

pub fn set_owner_group(target: &Path, uid: uid_t, gid: gid_t)
    -> Result<(), io::Error>
{
    let c_target = target.to_cstring();
    let rc = unsafe { lchown( c_target.as_ptr(), uid, gid) };
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
pub fn shallow_copy(src: &Path, src_stat: &Metadata, dest: &Path,
    owner_uid: Option<uid_t>, owner_gid: Option<gid_t>,
    mode: Option<u32>)
    -> Result<bool, io::Error>
{
    let src_type = src_stat.file_type();
    let mut is_dir = false;
    if src_type.is_dir() {
        is_dir = true;
        let nstat = dest.symlink_metadata();
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
            Dir::new(dest)
                .mode(mode.unwrap_or(src_stat.mode()))
                .uid(owner_uid.unwrap_or(src_stat.uid()))
                .gid(owner_gid.unwrap_or(src_stat.gid()))
                .create()?;
        }
    } else if src_type.is_symlink() {
        let value = fs::read_link(&src)?;
        force_symlink(&value, dest)?;
        set_owner_group(dest,
            owner_uid.unwrap_or(src_stat.uid()),
            owner_gid.unwrap_or(src_stat.gid()))?;
    } else {
        match mode {
            Some(mode) => copy_with_mode(src, dest, mode)?,
            None => copy(src, dest)?,
        }
        set_owner_group(dest,
            owner_uid.unwrap_or(src_stat.uid()),
            owner_gid.unwrap_or(src_stat.gid()))?;
    }
    Ok(!is_dir)
}

pub fn copy_utime<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q)
    -> io::Result<()>
{
    let metadata = fs::metadata(from.as_ref())?;
    let filename = CString::new(to.as_ref().as_os_str().as_bytes()).unwrap();
    let utimes = utimbuf {
        actime: metadata.atime(),
        modtime: metadata.mtime(),
    };
    let rc = unsafe { utime(filename.as_ptr(), &utimes) };
    if rc != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

pub fn safe_remove<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let path = path.as_ref();
    path.symlink_metadata()
        .and_then(|_| fs::remove_file(path))
        .or_else(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                Ok(())
            }
            else {
                Err(e)
            }
        })
}
