#![allow(dead_code)]
use std::ffi::CString;
use std::fs::{read_link};
use std::io::{ErrorKind, Error as IoError};
use std::path::{Path};
use std::ptr::null;

use libc::{c_ulong, c_int};
use libmount::{BindMount, Tmpfs};

use file_util::Dir;
use path_util::ToCString;

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

#[cfg(not(feature="containers"))]
pub fn unmount(target: &Path) -> Result<(), String> {
    unimplemented!();
}

#[cfg(feature="containers")]
pub fn unmount(target: &Path) -> Result<(), String> {
    let c_target = target.to_cstring();
    let rc = unsafe { umount2(c_target.as_bytes().as_ptr(), MNT_DETACH) };
    if rc == 0 {
        return Ok(());
    } else {
        let err = IoError::last_os_error();
        return Err(format!("Can't unmount {:?} : {}", target, err));
    }
}

pub fn mount_system_dirs() -> Result<(), String> {
    mount_dev(&Path::new("/vagga/root/dev"))?;
    BindMount::new("/sys", "/vagga/root/sys").mount()
        .map_err(|e| e.to_string())?;
    mount_proc(&Path::new("/vagga/root/proc"))?;
    BindMount::new("/work", "/vagga/root/work").mount()
        .map_err(|e| e.to_string())?;
    return Ok(())
}

pub fn unmount_system_dirs() -> Result<(), String> {
    unmount(Path::new("/vagga/root/work"))?;
    unmount(Path::new("/vagga/root/proc"))?;
    unmount(Path::new("/vagga/root/sys"))?;
    unmount(Path::new("/vagga/root/dev"))?;
    Ok(())
}

pub fn mount_dev(dev_dir: &Path) -> Result<(), String> {
    BindMount::new("/dev", &dev_dir).mount()
        .map_err(|e| e.to_string())?;

    let pts_dir = dev_dir.join("pts");
    mount_pseudo(&pts_dir, "devpts", "newinstance")?;

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
            BindMount::new(pts_dir.join("ptmx"), &ptmx_path).mount()
                .map_err(|e| e.to_string())?;
        }
        Err(e) => {
            warn!("Can't stat /dev/ptmx: {}. \
                Probably pseudo-ttys will not work", e);
        }
    }
    Ok(())
}

pub fn mount_run(run_dir: &Path) -> Result<(), String> {
    Tmpfs::new(&run_dir)
        .size_bytes(100 << 20)
        .mode(0o755)
        .mount().map_err(|e| format!("{}", e))?;
    try_msg!(Dir::new(&run_dir.join("shm")).mode(0o1777).create(),
        "Error creating /vagga/root/run/shm: {err}");
    Ok(())
}
