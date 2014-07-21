#![allow(dead_code)]

use std::io;
use std::ptr::null;
use std::c_str::CString;
use std::os::{errno, error_string};
use std::io::fs::mkdir;
use libc::{c_int, c_char, c_ulong, pid_t};
use libc::funcs::posix88::unistd::fork;

// errno.h
static EINTR: int = 4;
static ENOENT: int = 2;

// sched.h
static CLONE_NEWNS: c_int = 0x00020000;   /* Set to create new namespace.  */
static CLONE_NEWUTS: c_int = 0x04000000;  /* New utsname group.  */
static CLONE_NEWIPC: c_int = 0x08000000;  /* New ipcs.  */
static CLONE_NEWUSER: c_int = 0x10000000; /* New user namespace.  */
static CLONE_NEWPID: c_int = 0x20000000;  /* New pid namespace.  */
static CLONE_NEWNET: c_int = 0x40000000;  /* New network namespace.  */

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


extern  {
    // sched.h
    fn unshare(flags: c_int) -> c_int;

    // unistd.h
    fn chroot(dir: *c_char) -> c_int;
    fn execve(filename: *c_char, argv: **c_char, envp: **c_char) -> c_int;

    // sys/types.h
    // sys/wait.h
    fn waitpid(pid: pid_t, status: *c_int, options: c_int) -> pid_t;

    // sys/mount.h
    fn mount(source: *c_char, target: *c_char,
        filesystemtype: *c_char, flags: c_ulong,
        data: *c_char) -> c_int;

}

pub fn make_namespace() -> Result<(), String> {
    let rc = unsafe {
        unshare(CLONE_NEWNS|CLONE_NEWIPC|CLONE_NEWUSER|CLONE_NEWPID)
    };
    if rc != 0 {
        return Err(format!("Error making namespace: {}",
            error_string(errno() as uint)));
    }
    return Ok(());
}

pub fn change_root(root: &Path) -> Result<(), String> {
    debug!("Chrooting into {}", root.display());
    let rc = root.with_c_str(|root| unsafe { chroot(root) });
    if rc != 0 {
        return Err(format!("Error changing root: {}",
            error_string(errno() as uint)));
    }
    return Ok(());
}


pub fn bind_mount(source: &Path, target: &Path, read_write: bool)
    -> Result<(), String>
{
    debug!("Mounting {} into {}{}",
        source.display(), target.display(), if read_write {""} else {" (ro)"});
    let rc = unsafe {
        source.with_c_str(|source|
        target.with_c_str(|target|
            mount(source, target, null(), MS_BIND|MS_REC, null())
        ))};
    if rc != 0 {
        return Err(format!("Error mounting {}: {}",
            target.display(), error_string(errno() as uint)));
    }
    if !read_write {
        let rc = unsafe {
            source.with_c_str(|source|
            target.with_c_str(|target|
                mount(source, target, null(),
                      MS_BIND|MS_REMOUNT|MS_RDONLY, null())
            ))};
        if rc != 0 {
            return Err(format!("Error remounting ro {}: {}",
                target.display(), error_string(errno() as uint)));
        }
    }
    Ok(())
}

pub fn mount_pseudofs(fstype: &str, target: &Path, fsoptions: &str)
    -> Result<(), String>
{
    debug!("Mounting {} as {} with options: {}",
        target.display(), fstype, fsoptions);
    let flags = MS_NOSUID|MS_NODEV;
    let rc = unsafe {
        fstype.with_c_str(|fstype|
        target.with_c_str(|target|
        fsoptions.with_c_str(|fsoptions|
            mount(fstype, target, fstype, flags, fsoptions)
        )))};
    if rc != 0 {
        return Err(format!("Error mounting {}: {}",
            target.display(), error_string(errno() as uint)));
    }
    Ok(())
}

pub fn execute(command: &String, path: &[&str],
               args: &Vec<String>, environ: &Vec<String>)
    -> Result <(), String>
{
    debug!("Executing {} with args: {} with environ: {}",
        command, args, environ);
    let mut cargs: Vec<CString> = args.iter()
        .map(|s| { s.to_c_str() }).collect();
    cargs.insert(0, command.to_c_str());
    let cenviron: Vec<CString> = environ.iter()
        .map(|s| s.to_c_str()).collect();
    unsafe {
        let mut argv: Vec<*c_char> =
            cargs.iter().map(|s| s.with_ref(|p| p)).collect();
        argv.push(null());
        let mut envp: Vec<*c_char> =
            cenviron.iter().map(|s| s.with_ref(|p| p)).collect();
        envp.push(null());
        // TODO(tailhook) chdir
        command.with_c_str(|command|
            execve(
                command,
                argv.as_ptr(),
                envp.as_ptr(),
                ));
        if !command.as_slice().contains_char('/') && errno() == ENOENT {
            for prefix in path.iter() {
                (prefix.to_string() + "/" + *command).with_c_str(|command|
                    execve(
                        command,
                        argv.as_ptr(),
                        envp.as_ptr(),
                        ));
            }
        }
    }
    return Err(format!("Error executing command {}: {}",
        command, error_string(errno() as uint)));
}

pub fn forkme() -> Result<pid_t, String> {
    debug!("Forking");
    let rc = unsafe { fork() };
    if rc == -1 {
        return Err(format!("Error forking: {}",
            error_string(errno() as uint)));
    }
    return Ok(rc);
}

pub fn wait_process(pid: pid_t) -> Result<int, String> {
    loop {
        let status = 0;
        let rc = unsafe { waitpid(pid, &status, 0) };
        if rc < 0 {
            if errno() == EINTR {
                continue;
            } else {
                return Err(format!("Error waiting for child: {}",
                    error_string(errno() as uint)));
            }
        }
        return Ok(rc as int);
    }
}

pub fn ensure_dir(p: &Path) -> Result<(),String> {
    if p.exists() {
        return Ok(());
    }
    return mkdir(p, io::UserRWX).map_err(|e| { e.to_str() });
}
