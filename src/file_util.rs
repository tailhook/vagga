use std::io;
use std::io::{Read, Write, Error};
use std::path::{Path, PathBuf};
use std::fs;
use std::fs::{Metadata, rename};
use std::ffi::CString;
use std::time::{Instant, Duration};
use std::os::unix::fs::{PermissionsExt, MetadataExt, symlink};
use std::os::unix::io::AsRawFd;
use std::os::unix::ffi::OsStrExt;

use nix;
use libc;
use libc::{uid_t, gid_t, utime, utimbuf};
use nix::fcntl::{flock, FlockArg};
use digest_traits::Digest;
use sha2::Sha256;

use crate::digest::hex;
use crate::path_util::{tmp_file_name, ToCString};

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

pub enum CopyPolicy<T> {
    None,
    Preserve,
    Set(T),
}

impl<T> From<Option<T>> for CopyPolicy<T> {
    fn from(v: Option<T>) -> Self {
        match v {
            Some(v) => CopyPolicy::Set(v),
            None => CopyPolicy::None,
        }
    }
}

pub type CopyModePolicy = CopyPolicy<u32>;

pub type CopyOwnerPolicy = CopyPolicy<(u32, u32)>;

pub type CopyTimePolicy = CopyPolicy<(i64, i64)>;

pub struct FileCopy<'s, 'd> {
    src: &'s Path,
    dst: &'d Path,
    mode: CopyModePolicy,
    owner: CopyOwnerPolicy,
    time: CopyTimePolicy,
    atomic: bool,
}

impl<'s, 'd> FileCopy<'s, 'd> {
    pub fn new(src: &'s Path, dst: &'d Path) -> Self {
        Self {
            src,
            dst,
            mode: CopyPolicy::None,
            owner: CopyPolicy::None,
            time: CopyPolicy::None,
            atomic: false,
        }
    }

    pub fn time<T: Into<CopyTimePolicy>>(&mut self, time: T) -> &mut Self {
        self.time = time.into();
        self
    }

    pub fn owner<T: Into<CopyOwnerPolicy>>(&mut self, owner: T) -> &mut Self {
        self.owner = owner.into();
        self
    }

    pub fn mode<T: Into<CopyModePolicy>>(&mut self, mode: T) -> &mut Self {
        self.mode = mode.into();
        self
    }

    pub fn atomic(&mut self, atomic: bool) -> &mut Self {
        self.atomic = atomic;
        self
    }

    pub fn copy(&self) -> io::Result<()> {
        let _dst;
        let dst = if self.atomic {
            let name = match self.dst.file_name() {
                Some(name) => name,
                None => return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Cannot detect file name of {:?}", &self.dst)
                )),
            };
            let dst_dir = match self.dst.parent() {
                Some(dir) => dir,
                None => return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Cannot detect destination directory of {:?}", &self.dst)
                )),
            };
            _dst = dst_dir.join(tmp_file_name(name));
            _dst.as_path()
        } else {
            self.dst
        };

        let (mode, src_stat) = match self.mode {
            CopyModePolicy::None => (None, None),
            CopyModePolicy::Preserve => {
                let src_stat = self.src.symlink_metadata()?;
                (Some(src_stat.mode()), Some(src_stat))
            },
            CopyModePolicy::Set(mode) => {
                (Some(mode), None)
            },
        };

        _copy(&self.src, dst, mode)?;

        let src_stat = match self.owner {
            CopyOwnerPolicy::None => None,
            CopyOwnerPolicy::Preserve => {
                let src_stat = src_stat
                    .map_or_else(|| self.src.symlink_metadata(), Ok)?;
                set_owner_group(dst, src_stat.uid(), src_stat.gid())?;
                Some(src_stat)
            },
            CopyOwnerPolicy::Set((uid, gid)) => {
                set_owner_group(dst, uid, gid)?;
                None
            },
        };
        match self.time {
            CopyTimePolicy::None => {},
            CopyTimePolicy::Preserve => {
                let src_stat = src_stat
                    .map_or_else(|| self.src.symlink_metadata(), Ok)?;
                set_times(dst, src_stat.atime(), src_stat.mtime())?;
            },
            CopyTimePolicy::Set((atime, mtime)) => {
                set_times(dst, atime, mtime)?;
            },
        }

        if self.atomic {
            rename(dst, self.dst)?;
        }

        Ok(())
    }
}

pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<()>
{
    _copy(from.as_ref(), to.as_ref(), None)
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

pub fn copy_stream(reader: &mut dyn Read, writer: &mut dyn Write)
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
                nix::Error::InvalidUtf8 => unreachable!(),
                nix::Error::InvalidPath => unreachable!(),
                nix::Error::UnsupportedOperation => panic!("Can't flock"),
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
            Err(nix::Error::Sys(nix::errno::Errno::EAGAIN)) => {
                warn!("{}", message);
                let lock_start = Instant::now();
                flock(f.as_raw_fd(), FlockArg::LockExclusive)
                    .map_err(|e| match e {
                        nix::Error::Sys(code) => {
                            Error::from_raw_os_error(code as i32)
                        },
                        nix::Error::InvalidUtf8 => unreachable!(),
                        nix::Error::InvalidPath => unreachable!(),
                        nix::Error::UnsupportedOperation
                        => panic!("Can't flock"),
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
            Err(nix::Error::InvalidUtf8) => unreachable!(),
            Err(nix::Error::UnsupportedOperation) => panic!("Can't flock"),
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

pub fn check_stream_hashsum(mut reader: &mut dyn Read, sha256: &String)
    -> Result<(), String>
{
    let mut hash = Sha256::new();
    try_msg!(io::copy(&mut reader, &mut hash),
        "Error when calculating hashsum: {err}");
    let hash_str = format!("{:x}", hex(&hash));
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
    let rc = unsafe { libc::lchown( c_target.as_ptr(), uid, gid) };
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
pub struct ShallowCopy<'s, 'm, 'd> {
    src: &'s Path,
    src_stat: Option<&'m Metadata>,
    dst: &'d Path,
    owner_uid: Option<u32>,
    owner_gid: Option<u32>,
    mode: Option<u32>,
    time_policy: CopyTimePolicy,
}

impl<'s, 'm, 'd> ShallowCopy<'s, 'm, 'd> {
    pub fn new(src: &'s Path, dst: &'d Path)
        -> ShallowCopy<'s, 'm, 'd>
    {
        ShallowCopy {
            src: src,
            src_stat: None,
            dst: dst,
            owner_uid: None,
            owner_gid: None,
            mode: None,
            time_policy: CopyTimePolicy::None,
        }
    }

    pub fn src_stat(&mut self, stat: &'m Metadata)
        -> &mut ShallowCopy<'s, 'm, 'd>
    {
        self.src_stat = Some(stat);
        self
    }

    pub fn owner_uid<T: Into<Option<u32>>>(&mut self, uid: T)
        -> &mut ShallowCopy<'s, 'm, 'd>
    {
        self.owner_uid = uid.into();
        self
    }

    pub fn owner_gid<T: Into<Option<u32>>>(&mut self, gid: T)
        -> &mut ShallowCopy<'s, 'm, 'd>
    {
        self.owner_gid = gid.into();
        self
    }

    pub fn mode<T: Into<Option<u32>>>(&mut self, mode: T)
        -> &mut ShallowCopy<'s, 'm, 'd>
    {
        self.mode = mode.into();
        self
    }

    pub fn times(&mut self, atime: i64, mtime: i64)
        -> &mut ShallowCopy<'s, 'm, 'd>
    {
        self.time_policy = CopyTimePolicy::Set((atime, mtime));
        self
    }

    pub fn preserve_times(&mut self) -> &mut ShallowCopy<'s, 'm, 'd>
    {
        self.time_policy = CopyTimePolicy::Preserve;
        self
    }

    pub fn copy(&self) -> io::Result<bool> {
        shallow_copy(self.src, self.src_stat, self.dst,
            self.owner_uid, self.owner_gid, self.mode, &self.time_policy)
    }
}

fn shallow_copy(src: &Path, src_stat: Option<&Metadata>, dest: &Path,
    owner_uid: Option<uid_t>, owner_gid: Option<gid_t>,
    mode: Option<u32>, time_policy: &CopyTimePolicy)
    -> Result<bool, io::Error>
{
    let _src_stat;
    let src_stat = if let Some(stat) = src_stat {
        stat
    } else {
        _src_stat = src.symlink_metadata()?;
        &_src_stat
    };
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
        FileCopy::new(src, dest)
            .mode(mode)
            .owner(CopyOwnerPolicy::Set((
                owner_uid.unwrap_or(src_stat.uid()),
                owner_gid.unwrap_or(src_stat.gid())
            )))
            .copy()?;
        match *time_policy {
            CopyTimePolicy::None => {},
            CopyTimePolicy::Preserve => {
                set_times(dest, src_stat.atime(), src_stat.mtime())?;
            },
            CopyTimePolicy::Set((atime, mtime)) => {
                set_times(dest, atime, mtime)?;
            },
        }
    }
    Ok(!is_dir)
}

pub fn set_times<P: AsRef<Path>>(path: P, atime: i64, mtime: i64)
    -> io::Result<()>
{
    let filename = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
    let utimes = utimbuf {
        actime: atime,
        modtime: mtime,
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

pub fn truncate_file<P: AsRef<Path>>(path: P) -> Result<(), String> {
    let path = path.as_ref();
    if path.symlink_metadata()
        .map(|m| m.file_type().is_file())
        .or_else(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                Ok(true)
            } else {
                Err(format!("Cannot stat {:?} file: {}", path, e))
            }
        })?
    {
        fs::File::create(path).map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                format!(
                    "Cannot create file {:?}: no such directory {:?}",
                    path, path.parent())
            } else {
                format!("Cannot truncate file {:?}: {}", path, e)
            }
        })?;
    }
    Ok(())
}

pub fn human_size(size: u64) -> String {
    fn format_size(s: f64, p: &str) -> String {
        if s < 10.0 && !p.is_empty() {
            format!("{:.1}{}B", s, p)
        } else {
            format!("{:.0}{}B", s, p)
        }
    }

    let mut s = size as f64;
    for prefix in &["", "K", "M", "G", "T"][..] {
        if s < 1000.0 {
            return format_size(s, prefix);
        }
        s /= 1000.0;
    }
    return format_size(s, "P");
}
