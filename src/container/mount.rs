#![allow(dead_code)]
use std::ffi::CString;
use std::fs::{File, read_link};
use std::io::{ErrorKind, Error as IoError};
use std::io::{BufRead, BufReader};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::ptr::null;
use std::str::FromStr;

use libc::{c_ulong, c_int};

use super::super::path_util::{ToCString, ToRelative};
use super::tools::NextValue;

// sys/mount.h
static MS_RDONLY: c_ulong = 1;                /* Mount read-only.  */
static MS_NOSUID: c_ulong = 2;                /* Ignore suid and sgid bits.  */
static MS_NODEV: c_ulong = 4;                 /* Disallow access to device special files.  */
static MS_NOEXEC: c_ulong = 8;                /* Disallow program execution.  */
static MS_SYNCHRONOUS: c_ulong = 16;          /* Writes are synced at once.  */
static MS_REMOUNT: c_ulong = 32;              /* Alter flags of a mounted FS.  */
static MS_MANDLOCK: c_ulong = 64;             /* Allow mandatory locks on an FS.  */
static MS_DIRSYNC: c_ulong = 128;             /* Directory modifications are synchronous.  */
static MS_NOATIME: c_ulong = 1024;            /* Do not update access times.  */
static MS_NODIRATIME: c_ulong = 2048;         /* Do not update directory access times.  */
static MS_BIND: c_ulong = 4096;               /* Bind directory at different place.  */
static MS_MOVE: c_ulong = 8192;
static MS_REC: c_ulong = 16384;
static MS_SILENT: c_ulong = 32768;
static MS_POSIXACL: c_ulong = 1 << 16;        /* VFS does not apply the umask.  */
static MS_UNBINDABLE: c_ulong = 1 << 17;      /* Change to unbindable.  */
static MS_PRIVATE: c_ulong = 1 << 18;         /* Change to private.  */
static MS_SLAVE: c_ulong = 1 << 19;           /* Change to slave.  */
static MS_SHARED: c_ulong = 1 << 20;          /* Change to shared.  */
static MS_RELATIME: c_ulong = 1 << 21;        /* Update atime relative to mtime/ctime.  */
static MS_KERNMOUNT: c_ulong = 1 << 22;       /* This is a kern_mount call.  */
static MS_I_VERSION: c_ulong =  1 << 23;      /* Update inode I_version field.  */
static MS_STRICTATIME: c_ulong = 1 << 24;     /* Always perform atime updates.  */
static MS_ACTIVE: c_ulong = 1 << 30;
static MS_NOUSER: c_ulong = 1 << 31;

static MNT_FORCE: c_int = 1;           /* Force unmounting.  */
static MNT_DETACH: c_int = 2;          /* Just detach from the tree.  */
static MNT_EXPIRE: c_int = 4;          /* Mark for expiry.  */
static UMOUNT_NOFOLLOW: c_int = 8;     /* Don't follow symlink on umount.  */

extern {
    fn mount(source: *const u8, target: *const u8,
        filesystemtype: *const u8, flags: c_ulong,
        data: *const u8) -> c_int;
    fn umount(target: *const u8) -> c_int;
    fn umount2(target: *const u8, flags: c_int) -> c_int;
}


pub struct MountRecord<'a> {
    pub mount_id: usize,
    pub parent_id: usize,
    _device: &'a str,  // TODO(tailhook) parse if ever need
    pub relative_root: &'a str,
    pub mount_point: &'a str,
    pub mount_options: &'a str,
    pub tag_shared: Option<usize>,
    pub tag_master: Option<usize>,
    pub tag_propagate_from: Option<usize>,
    pub tag_unbindable: Option<()>,
    pub fstype: &'a str,
    pub mount_source: &'a str,
    pub super_options: &'a str,
}

impl<'a> MountRecord<'a> {
    pub fn from_str<'x>(line: &'x str) -> Result<MountRecord<'x>, ()> {
        let mut parts = line.split_whitespace();
        let mount_id = try!(parts.next_value());
        let parent_id = try!(parts.next_value());
        let device = try!(parts.next().ok_or(()));
        let relative_root = try!(parts.next().ok_or(()));
        let mount_point = try!(parts.next().ok_or(()));
        let mount_options = try!(parts.next().ok_or(()));
        let mut tag_shared = None;
        let mut tag_master = None;
        let mut tag_propagate_from = None;
        let mut tag_unbindable = None;

        for name in &mut parts {
            if name == "-" { break; }
            let mut pair = name.splitn(2, ':');
            let key = pair.next();
            match key {
                Some("shared") => {
                    tag_shared = Some(try!(pair.next_value()));
                }
                Some("master") => {
                    tag_master = Some(try!(pair.next_value()));
                }
                Some("propagate_from") => {
                    tag_propagate_from = Some(try!(pair.next_value()));
                }
                Some("unbindable") => tag_unbindable = Some(()),
                _ => {}
            }
        }

        let fstype = try!(parts.next().ok_or(()));
        let mount_source = try!(parts.next().ok_or(()));
        let super_options = try!(parts.next().ok_or(()));

        return Ok(MountRecord {
            mount_id: mount_id,
            parent_id: parent_id,
            _device: device,
            relative_root: relative_root,
            mount_point: mount_point,
            mount_options: mount_options,
            tag_shared: tag_shared,
            tag_master: tag_master,
            tag_propagate_from: tag_propagate_from,
            tag_unbindable: tag_unbindable,
            fstype: fstype,
            mount_source: mount_source,
            super_options: super_options,
            });
    }
    pub fn is_private(&self) -> bool {
        return self.tag_shared.is_none()
            && self.tag_master.is_none()
            && self.tag_propagate_from.is_none()
            && self.tag_unbindable.is_none();
    }
}

pub fn get_submounts_of(dir: &Path)
    -> Result<Vec<PathBuf>, String>
{
    let f = try!(File::open(&Path::new("/proc/self/mountinfo"))
        .map_err(|e| format!("Can't open mountinfo: {}", e)));
    let buf = BufReader::new(f);
    let mut result = vec!();
    for line in buf.lines() {
        let line = try!(line
            .map_err(|e| format!("Can't read mountinfo: {}", e)));
        match MountRecord::from_str(&line) {
            Ok(rec) => {
                let path = Path::new(rec.mount_point);
                if dir.is_ancestor(&path) {
                    result.push(path.to_path_buf());
                }
            }
            Err(()) => {
                return Err(format!("Can't parse mountinfo line: {}", line));
            }
        }
    }
    return Ok(result);
}

pub fn remount_ro(target: &Path) -> Result<(), String> {
    let none = CString::new("none").unwrap();
    debug!("Remount readonly: {:?}", target);
    let c_target = target.to_cstring();
    let rc = unsafe { mount(
       none.as_bytes().as_ptr(),
       c_target.as_bytes().as_ptr(),
       null(), MS_BIND|MS_REMOUNT|MS_RDONLY, null()) };
    if rc != 0 {
        let err = IoError::last_os_error();
        return Err(format!("Remount readonly {:?}: {}", target, err));
    }
    return Ok(());
}

pub fn mount_private(target: &Path) -> Result<(), String> {
    let none = CString::new("none").unwrap();
    let c_target = target.to_cstring();
    debug!("Making private {:?}", target);
    let rc = unsafe { mount(
        none.as_bytes().as_ptr(),
        c_target.as_bytes().as_ptr(),
        null(), MS_PRIVATE, null()) };
    if rc == 0 {
        return Ok(());
    } else {
        let err = IoError::last_os_error();
        return Err(format!("Can't make {:?} a slave: {}", target, err));
    }
}

pub fn bind_mount(source: &Path, target: &Path) -> Result<(), String> {
    let c_source = source.to_cstring();
    let c_target = target.to_cstring();
    debug!("Bind mount {:?} -> {:?}", source, target);
    let rc = unsafe {
        mount(c_source.as_bytes().as_ptr(), c_target.as_bytes().as_ptr(),
        null(), MS_BIND|MS_REC, null()) };
    if rc == 0 {
        return Ok(());
    } else {
        let err = IoError::last_os_error();
        return Err(format!("Can't mount bind {:?} to {:?}: {}",
            source, target, err));
    }
}

pub fn mount_proc(target: &Path) -> Result<(), String>
{
    let c_target = target.to_cstring();

    // I don't know why we need this flag for mounting proc, but it's somehow
    // works (see https://github.com/tailhook/vagga/issues/12 for the
    // error it fixes)
    let flags = 0xC0ED0000; //MS_MGC_VAL

    debug!("Procfs mount {:?}", target);
    let rc = unsafe { mount(
        b"proc\x00".as_ptr(), c_target.as_bytes().as_ptr(),
        b"proc\x00".as_ptr(), flags, null()) };
    if rc == 0 {
        return Ok(());
    } else {
        let err = IoError::last_os_error();
        return Err(format!("Can't mount proc at {:?}: {}",
            target, err));
    }
}

pub fn mount_pseudo(target: &Path, name: &str, options: &str)
    -> Result<(), String>
{
    let c_name = name.to_cstring();
    let c_target = target.to_cstring();
    let c_opts = options.to_cstring();
    // Seems this is similar to why proc is mounted with the flag
    let flags = 0xC0ED0000; //MS_MGC_VAL

    debug!("Pseusofs mount {:?} {} {}", target, name, options);
    let rc = unsafe { mount(
        c_name.as_bytes().as_ptr(),
        c_target.as_bytes().as_ptr(),
        c_name.as_bytes().as_ptr(),
        flags,
        c_opts.as_bytes().as_ptr()) };
    if rc == 0 {
        return Ok(());
    } else {
        let err = IoError::last_os_error();
        return Err(format!("Can't mount pseudofs {:?} ({}, options: {}): {}",
            target, options, name, err));
    }
}

pub fn mount_tmpfs(target: &Path, options: &str) -> Result<(), String> {
    let c_tmpfs = CString::new("tmpfs").unwrap();
    let c_target = target.to_cstring();
    let c_opts = options.to_cstring();
    debug!("Tmpfs mount {:?} {}", target, options);
    let rc = unsafe { mount(
        c_tmpfs.as_bytes().as_ptr(),
        c_target.as_bytes().as_ptr(),
        c_tmpfs.as_bytes().as_ptr(),
        MS_NOSUID | MS_NODEV | MS_NOATIME,
        c_opts.as_bytes().as_ptr()) };
    if rc == 0 {
        return Ok(());
    } else {
        let err = IoError::last_os_error();
        return Err(format!("Can't mount tmpfs {:?} (options: {}): {}",
            target, options, err));
    }
}

pub fn unmount(target: &Path) -> Result<(), String> {
    let c_target = target.to_cstring();
    let rc = unsafe { umount2(c_target.as_bytes().as_ptr(), MNT_DETACH) };
    if rc == 0 {
        return Ok(());
    } else {
        let err = IoError::last_os_error();
        return Err(format!("Can't unmount {} : {}", target.display(), err));
    }
}

pub fn mount_system_dirs() -> Result<(), String> {
    try!(mount_dev(&Path::new("/vagga/root/dev")));
    try!(bind_mount(&Path::new("/sys"), &Path::new("/vagga/root/sys")));
    try!(mount_proc(&Path::new("/vagga/root/proc")));
    try!(bind_mount(&Path::new("/work"), &Path::new("/vagga/root/work")));
    return Ok(());
}

pub fn mount_dev(dev_dir: &Path) -> Result<(), String> {
    try!(bind_mount(&Path::new("/dev"), &dev_dir));

    let pts_dir = dev_dir.join("pts");
    try!(mount_pseudo(&pts_dir, "devpts", "newinstance"));

    let ptmx_path = dev_dir.join("ptmx");
    match read_link(&ptmx_path) {
        Ok(x) => {
            if Path::new(&x) != Path::new("/dev/pts/ptmx")
               && Path::new(&x) != Path::new("pts/ptmx")
            {
                warn!("The /dev/ptmx refers to {:?}. We need /dev/pts/ptmx \
                    to operate properly. \
                    Probably pseudo-ttys will not work", x);
            }
        }
        Err(ref e) if e.kind() == ErrorKind::InvalidInput => {
            // It's just a device. Let's try bind mount
            try!(bind_mount(&pts_dir.join("ptmx"), &ptmx_path));
        }
        Err(e) => {
            warn!("Can't stat /dev/ptmx: {}. \
                Probably pseudo-ttys will not work", e);
        }
    }
    Ok(())
}
