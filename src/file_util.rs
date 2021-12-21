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

#[derive(Clone, Copy)]
enum CopyPolicy<T: Copy> {
    None,
    Preserve,
    Set(T),
}

trait ResolveCopyPolicy<T: Copy>: Into<CopyPolicy<T>> + Copy {
    fn metadata_value(&self, stat: &Metadata) -> T;

    fn resolve(
        &self, src: &Path, src_stat: Option<&Metadata>
    ) -> io::Result<(Option<T>, Option<Metadata>)> {
        match (*self).into() {
            CopyPolicy::None => Ok((None, None)),
            CopyPolicy::Preserve => {
                match src_stat {
                    Some(s) => Ok((Some(self.metadata_value(&s)), None)),
                    None => {
                        let stat = src.symlink_metadata()?;
                        Ok((Some(self.metadata_value(&stat)), Some(stat)))
                    },
                }
            }
            CopyPolicy::Set(v) => Ok((Some(v), None)),
        }
    }
}

#[derive(Clone, Copy)]
pub enum CopyModePolicy {
    None,
    Set(u32),
}

impl ResolveCopyPolicy<u32> for CopyModePolicy {
    fn metadata_value(&self, stat: &Metadata) -> u32 {
        stat.mode()
    }
}

impl From<CopyModePolicy> for CopyPolicy<u32> {
    fn from(p: CopyModePolicy) -> Self {
        use CopyModePolicy::*;
        match p {
            None => Self::None,
            Set(mode) => Self::Set(mode),
        }
    }
}

impl From<Option<u32>> for CopyModePolicy {
    fn from(v: Option<u32>) -> Self {
        match v {
            Some(v) => CopyModePolicy::Set(v),
            None => CopyModePolicy::None,
        }
    }
}

#[derive(Clone, Copy)]
pub enum CopyOwnerPolicy {
    Preserve,
    Set(u32),
}

impl From<Option<u32>> for CopyOwnerPolicy {
    fn from(v: Option<u32>) -> Self {
        match v {
            Some(v) => Self::Set(v),
            None => Self::Preserve,
        }
    }
}

#[derive(Clone, Copy)]
enum CopyOwnerPolicyWrapper {
    Uid(CopyOwnerPolicy),
    Gid(CopyOwnerPolicy),
}

impl ResolveCopyPolicy<u32> for CopyOwnerPolicyWrapper {
    fn metadata_value(&self, stat: &Metadata) -> u32 {
        match self {
            Self::Uid(_) => stat.uid(),
            Self::Gid(_) => stat.gid(),
        }
    }
}

impl From<CopyOwnerPolicyWrapper> for CopyPolicy<u32> {
    fn from(p: CopyOwnerPolicyWrapper) -> Self {
        use CopyOwnerPolicyWrapper::*;
        let policy = match p {
            Uid(p) => p,
            Gid(p) => p,
        };
        match policy {
            CopyOwnerPolicy::Preserve => Self::Preserve,
            CopyOwnerPolicy::Set(id) => Self::Set(id),
        }
    }
}

#[derive(Clone, Copy)]
pub enum CopyTimePolicy {
    None,
    Preserve,
    Set { atime: i64, mtime: i64 }
}

impl ResolveCopyPolicy<(i64, i64)> for CopyTimePolicy {
    fn metadata_value(&self, stat: &Metadata) -> (i64, i64) {
        (stat.atime(), stat.mtime())
    }
}

impl From<CopyTimePolicy> for CopyPolicy<(i64, i64)> {
    fn from(p: CopyTimePolicy) -> Self {
        use CopyTimePolicy::*;
        match p {
            None => Self::None,
            Preserve => Self::Preserve,
            Set { atime, mtime } => Self::Set((atime, mtime)),
        }
    }
}

pub struct FileCopy<'s, 'd> {
    src: &'s Path,
    dst: &'d Path,
    src_stat: Option<&'s Metadata>,
    mode: CopyModePolicy,
    owner_uid: CopyOwnerPolicyWrapper,
    owner_gid: CopyOwnerPolicyWrapper,
    time: CopyTimePolicy,
    atomic: bool,
}

impl<'s, 'd> FileCopy<'s, 'd> {
    pub fn new(src: &'s Path, dst: &'d Path) -> Self {
        Self {
            src,
            dst,
            src_stat: None,
            mode: CopyModePolicy::None,
            owner_uid: CopyOwnerPolicyWrapper::Uid(CopyOwnerPolicy::Preserve),
            owner_gid: CopyOwnerPolicyWrapper::Gid(CopyOwnerPolicy::Preserve),
            time: CopyTimePolicy::None,
            atomic: false,
        }
    }

    pub fn time<T: Into<CopyTimePolicy>>(&mut self, time: T) -> &mut Self {
        self.time = time.into();
        self
    }

    pub fn owner_uid<T: Into<CopyOwnerPolicy>>(&mut self, owner: T) -> &mut Self {
        self.owner_uid = CopyOwnerPolicyWrapper::Uid(owner.into());
        self
    }

    pub fn owner_gid<T: Into<CopyOwnerPolicy>>(&mut self, owner: T) -> &mut Self {
        self.owner_gid = CopyOwnerPolicyWrapper::Gid(owner.into());
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

        let (mode, src_stat) = self.mode.resolve(
            self.src, self.src_stat
        )?;
        _copy(&self.src, dst, mode)?;

        let (uid, src_stat) = self.owner_uid.resolve(
            self.src, src_stat.as_ref().or(self.src_stat)
        )?;
        let (gid, src_stat) = self.owner_gid.resolve(
            self.src, src_stat.as_ref().or(self.src_stat)
        )?;
        set_owner_group(dst, uid.expect("Owner uid"), gid.expect("Owner gid"))?;

        let (time, _) = self.time.resolve(
            self.src, src_stat.as_ref().or(self.src_stat)
        )?;
        if let Some((atime, mtime)) = time {
            set_times(dst, atime, mtime)?;
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
        self.time_policy = CopyTimePolicy::Set { atime, mtime };
        self
    }

    pub fn preserve_times(&mut self) -> &mut ShallowCopy<'s, 'm, 'd>
    {
        self.time_policy = CopyTimePolicy::Preserve;
        self
    }

    pub fn copy(&self) -> io::Result<bool> {
        shallow_copy(self.src, self.src_stat, self.dst,
            self.owner_uid, self.owner_gid, self.mode, self.time_policy)
    }
}

fn shallow_copy(src: &Path, src_stat: Option<&Metadata>, dest: &Path,
    owner_uid: Option<uid_t>, owner_gid: Option<gid_t>,
    mode: Option<u32>, time_policy: CopyTimePolicy)
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
            .time(time_policy)
            .owner_uid(owner_uid)
            .owner_gid(owner_gid)
            .copy()?;
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
