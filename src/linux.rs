#![allow(dead_code)]
#![allow(non_camel_case_types)]

use std::ptr::null;
use std::c_str::CString;
use std::string::raw::from_buf;
use std::os::{errno, error_string};
use std::io::ALL_PERMISSIONS;
use std::io::fs::mkdir;
use std::io::fs::PathExtensions;
use std::os::{Pipe, pipe};
use std::os::getenv;
use std::default::Default;
use serialize::{Decoder, Decodable};
use libc::{c_int, c_uint, c_char, c_ulong, pid_t, _exit, c_void, uid_t, gid_t};
use libc::funcs::posix88::unistd::{close, write};
use libc::consts::os::posix88::{EINTR, EAGAIN, EINVAL};

use collections::treemap::TreeMap;

use super::env::Environ;

// sys/types.h
// sys/wait.h
static WNOHANG: c_int = 1;

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

// signal.h
static SIG_BLOCK: c_int = 0;
static SIG_UNBLOCK: c_int = 1;
static SIG_SETMASK: c_int = 2;

// sys/prctl.h
static PR_SET_CHILD_SUBREAPER: c_int = 36;

pub static SIGCHLD   : c_int =  17    ; /* Child status has changed (POSIX).  */

// pwd.h
#[repr(C)]
struct passwd {
    pw_name: *mut c_char,       /* username */
    pw_passwd: *mut u8,     /* user password */
    pw_uid: uid_t,      /* user ID */
    pw_gid: gid_t,      /* group ID */
    pw_gecos: *mut u8,      /* user information */
    pw_dir: *mut u8,        /* home directory */
    pw_shell: *mut u8,      /* shell program */
}

extern  {
    // sys/types.h
    // sys/wait.h
    fn waitpid(pid: pid_t, status: *mut c_int, options: c_int) -> pid_t;

    // sys/mount.h
    fn mount(source: *const c_char, target: *const c_char,
        filesystemtype: *const c_char, flags: c_ulong,
        data: *const c_char) -> c_int;

    // signal.h
    fn sigprocmask(how: c_int, set: *const u8, oldset: *mut u8) -> c_int;
    fn sigwait(set: *const u8, sig: *mut c_int) -> c_int;
    fn sigfillset(set: *mut u8) -> c_int;

    // sys/prctl.h
    fn prctl(option: c_int, arg2: c_ulong, arg3: c_ulong,
                            arg4: c_ulong, arg5: c_ulong) -> c_int;


    // pwd.h
    fn getpwuid(uid: uid_t) -> *const passwd;

}

#[deriving(Clone)]
pub enum Mount {
    Bind(CString, CString),
    BindRO(CString, CString),
    BindROTmp(CString, CString),
    Pseudo(CString, CString, CString),
}

#[deriving(Clone, PartialEq, Eq)]
pub enum Pid1Mode {
    Exec = 0,
    Wait = 1,
    WaitAllChildren = 2,
}

impl<E, D:Decoder<E>> Decodable<D, E> for Pid1Mode {
    fn decode(d: &mut D) -> Result<Pid1Mode, E> {
        d.read_enum("Pid1Mode", |d| {
            d.read_enum_variant(["exec", "wait", "wait-all-children"], |_, i| {
                Ok(match i {
                    0 => Exec,
                    1 => Wait,
                    2 => WaitAllChildren,
                    _ => unreachable!(),
                })
            })
        })
    }
}

pub enum ExtFlags {
    FlagMkdir = 1,
}

/* Keep in sync with container.c */
#[repr(C)]
struct CMount {
    source: *const u8,
    target: *const u8,
    fstype: *const u8,
    options: *const u8,
    flags: c_ulong,
    ext_flags: c_uint,
}

/* Keep in sync with container.c */
#[repr(C)]
struct CContainer {
    pid1_mode: c_int,
    pipe_reader: c_int,
    pipe_writer: c_int,
    container_root: *const u8,
    mount_dir: *const u8,
    pivot_dir: *const u8,
    mounts_num: c_int,
    mounts: *const CMount,
    work_dir: *const u8,
    exec_filename: *const u8,     // For error messages
    exec_filenames_num: c_int,
    exec_filenames: *const *const u8,   // Full paths to try in order
    exec_args: *const *const u8,
    exec_environ: *const *const u8,
}


pub struct RunOptions {
    pub writeable: bool,
    pub uidmap: bool,
    pub pid1mode: Pid1Mode,
    pub mounts: Vec<Mount>,
}

impl Default for RunOptions {
    fn default() -> RunOptions {
        return RunOptions {
            writeable: false,
            uidmap: getenv("VAGGA_IN_BUILD").is_none(),
            pid1mode: Wait,
            mounts: Vec::new(),
        };
    }
}

pub struct CPipe(Pipe);

impl CPipe {
    pub fn new() -> Result<CPipe, String> {
        unsafe {
            match pipe() {
                Ok(pipe) => Ok(CPipe(pipe)),
                Err(e) => Err(format!("Can't create pipe: {}", e)),
            }
        }
    }
    pub fn wakeup(&self) -> Result<(), String> {
        let mut rc;
        let &CPipe(ref pipe) = self;
        loop {
            unsafe {
                rc = write(pipe.writer, ['x' as u8].as_ptr() as *const c_void, 1);
            }
            if rc < 0 && (errno() as i32 == EINTR || errno() as i32 == EAGAIN) {
                continue
            }
            break;
        }
        if rc == 0 {
            return Err(format!("File already closed"));
        } else if rc == -1 {
            return Err(format!("Error writing to pipe: {}",
                error_string(errno() as uint)));
        }
        return Ok(());
    }
    fn drop(&self) {
        match self {
            &CPipe(ref pipe) => {
                unsafe {
                    close(pipe.reader);
                    close(pipe.writer);
                }
            }
        }
    }
}

#[link(name="container", kind="static")]
extern  {
    fn fork_to_container(flags: c_int, container: *const CContainer) -> pid_t;
}

impl Mount {
    fn to_c_mount(&self) -> CMount {
        match self {
            &Bind(ref a, ref b) => CMount {
                source: a.as_bytes().as_ptr(),
                target: b.as_bytes().as_ptr(),
                fstype: null(),
                options: null(),
                flags: MS_BIND|MS_REC,
                ext_flags: 0,
            },
            &BindRO(ref a, ref b) => CMount {
                source: a.as_bytes().as_ptr(),
                target: b.as_bytes().as_ptr(),
                fstype: null(),
                options: null(),
                flags: MS_BIND|MS_REC|MS_RDONLY,
                ext_flags: 0,
            },
            &BindROTmp(ref a, ref b) => CMount {
                source: a.as_bytes().as_ptr(),
                target: b.as_bytes().as_ptr(),
                fstype: null(),
                options: null(),
                flags: MS_BIND|MS_REC|MS_RDONLY,
                ext_flags: FlagMkdir as c_uint,
            },
            &Pseudo(ref fs, ref dir, ref options) => CMount {
                source: fs.as_bytes().as_ptr(),
                target: dir.as_bytes().as_ptr(),
                fstype: fs.as_bytes().as_ptr(),
                options: options.as_bytes().as_ptr(),
                flags: MS_NOSUID|MS_NODEV,
                ext_flags: 0,
            },
        }
    }
}

fn c_vec<'x, T:ToCStr, I:Iterator<&'x T>>(iter: I) -> Vec<CString> {
    return iter.map(|a| a.to_c_str()).collect();
}

fn raw_vec(vec: &Vec<CString>) -> Vec<*const u8> {
    return vec.iter().map(|a| a.as_bytes().as_ptr()).collect();
}

pub fn run_container(pipe: &CPipe, env: &Environ, root: &Path,
    options: &RunOptions, work_dir: &Path,
    cmd: &String, args: &[String], environ: &TreeMap<String, String>)
    -> Result<pid_t, String>
{
    let c_container_root = root.to_c_str();
    let mount_dir = env.local_vagga.join(".mnt");
    try!(ensure_dir(&mount_dir));
    let c_mount_dir = mount_dir.to_c_str();
    let c_pivot_dir = mount_dir.join("mnt").to_c_str();
    // TODO(pc) find recursive bindings for BindRO
    let rootmount = if options.writeable {
        Bind(root.to_c_str(), mount_dir.to_c_str())
    } else {
        BindRO(root.to_c_str(), mount_dir.to_c_str())
    };
    let mut mounts = vec!(
        rootmount,
        BindRO("/sys".to_c_str(), mount_dir.join("sys").to_c_str()),
        // TODO(tailhook) use dev in /var/lib/container-dev
        BindRO("/dev".to_c_str(), mount_dir.join("dev").to_c_str()),
        Bind(env.project_root.to_c_str(), mount_dir.join("work").to_c_str()),
        Pseudo("proc".to_c_str(), mount_dir.join("proc").to_c_str(),
            "".to_c_str()),
        );
    mounts.extend(options.mounts.clone().into_iter());
    let c_mounts: Vec<CMount> = mounts.iter().map(|v| v.to_c_mount()).collect();

    let c_work_dir = match work_dir.path_relative_from(&env.project_root) {
        Some(path) => Path::new("/work").join(path).to_c_str(),
        None => "/work".to_c_str(),
    };
    let c_exec_fn = cmd.to_c_str();
    let filenames = if cmd.as_slice().contains("/") {
            vec!(c_exec_fn.clone())
        } else {
            environ.find(&"PATH".to_string()).unwrap()
                .as_slice().split(':').map(|prefix| {
                    (prefix.to_string() + "/".to_string() + cmd.to_string())
                    .to_c_str()
                }).collect()
        };
    let c_filenames:Vec<*const u8> = raw_vec(&filenames);
    let args = c_vec(args.iter());
    let mut c_args = raw_vec(&args);
    c_args.insert(0, c_exec_fn.as_bytes().as_ptr());
    c_args.push(null());
    let environ = environ.iter().map(|(k, v)| {
        (*k + "=" + *v).to_c_str()
    }).collect();
    let mut c_environ = raw_vec(&environ);
    c_environ.push(null());

    let &CPipe(pipe) = pipe;
    let mut ns = CLONE_NEWPID|CLONE_NEWNS|CLONE_NEWIPC;
    if options.uidmap {
        ns |= CLONE_NEWUSER;
    }
    let pid = unsafe {
        fork_to_container(ns,
            &CContainer {
                pid1_mode: options.pid1mode as i32,
                pipe_reader: pipe.reader,
                pipe_writer: pipe.writer,
                container_root: c_container_root.as_bytes().as_ptr(),
                mount_dir: c_mount_dir.as_bytes().as_ptr(),
                pivot_dir: c_pivot_dir.as_bytes().as_ptr(),
                mounts_num: c_mounts.len() as i32,
                mounts: c_mounts.as_slice().as_ptr(),
                work_dir: c_work_dir.as_bytes().as_ptr(),
                exec_filename: c_exec_fn.as_bytes().as_ptr(),
                exec_filenames_num: c_filenames.len() as i32,
                exec_filenames: c_filenames.as_slice().as_ptr(),
                exec_args: c_args.as_slice().as_ptr(),
                exec_environ: c_environ.as_slice().as_ptr(),
            })
    };
    if pid < 0 {
        let eno = errno() as i32;
        let err = error_string(eno as uint);
        if eno == EINVAL {
            return Err(format!("Error cloning: {}. It might mean that your
                kernel doesn't support user namespaces. Sorry.", err));
        } else {
            return Err(format!("Error cloning: {}", err));
        }
    }
    return Ok(pid);
}

pub fn run_newuser(pipe: &CPipe,
    cmd: &String, args: &[String], environ: &TreeMap<String, String>)
    -> Result<pid_t, String>
{
    let c_exec_fn = cmd.to_c_str();
    let filenames = if cmd.as_slice().contains("/") {
            vec!(c_exec_fn.clone())
        } else {
            environ.find(&"PATH".to_string()).unwrap()
                .as_slice().split(':').map(|prefix| {
                    (prefix.to_string() + "/".to_string() + cmd.to_string())
                    .to_c_str()
                }).collect()
        };
    let c_filenames:Vec<*const u8> = raw_vec(&filenames);
    let args = c_vec(args.iter());
    let mut c_args = raw_vec(&args);
    c_args.insert(0, c_exec_fn.as_bytes().as_ptr());
    c_args.push(null());
    let environ = environ.iter().map(|(k, v)| {
        (*k + "=" + *v).to_c_str()
    }).collect();
    let mut c_environ = raw_vec(&environ);
    c_environ.push(null());

    let &CPipe(pipe) = pipe;
    let pid = unsafe {
        fork_to_container(
            CLONE_NEWUSER,
            &CContainer {
                pid1_mode: Exec as i32,
                pipe_reader: pipe.reader,
                pipe_writer: pipe.writer,
                container_root: null(),
                mount_dir: null(),
                pivot_dir: null(),
                mounts_num: 0,
                mounts: null(),
                work_dir: null(),
                exec_filename: c_exec_fn.as_bytes().as_ptr(),
                exec_filenames_num: c_filenames.len() as i32,
                exec_filenames: c_filenames.as_slice().as_ptr(),
                exec_args: c_args.as_slice().as_ptr(),
                exec_environ: c_environ.as_slice().as_ptr(),
            })
    };
    if pid < 0 {
        let eno = errno() as i32;
        let err = error_string(eno as uint);
        if eno == EINVAL {
            return Err(format!("Error cloning: {}. It might mean that your
                kernel doesn't support user namespaces. Sorry.", err));
        } else {
            return Err(format!("Error cloning: {}", err));
        }
    }
    return Ok(pid);
}


pub fn wait_process(pid: pid_t) -> Result<int, String> {
    loop {
        let mut status: i32 = 0;
        let rc = unsafe { waitpid(pid, &mut status, 0) };
        if rc < 0 {
            if errno() as i32 == EINTR {
                continue;
            } else {
                return Err(format!("Error waiting for child: {}",
                    error_string(errno() as uint)));
            }
        }
        assert_eq!(rc, pid);
        info!("Process {} exited with {}", pid, status);
        return Ok(status as int);
    }
}

struct DeadProcesses;

impl Iterator<(pid_t, i32)> for DeadProcesses {
    fn next(&mut self) -> Option<(pid_t, i32)> {
        loop {
            let mut status = 0;
            let pid = unsafe { waitpid(-1, &mut status, WNOHANG) };
            if pid == 0 {
                return None;
            }
            if pid < 0 {
                debug!("Error in waitpid: {}", error_string(errno() as uint));
                continue;
            }
            return Some((pid, status));
        }
    }
}

pub fn dead_processes() -> DeadProcesses { DeadProcesses }

pub fn ensure_dir(p: &Path) -> Result<(),String> {
    if p.exists() {
        return Ok(());
    }
    return mkdir(p, ALL_PERMISSIONS).map_err(|e| { e.to_string() });
}

pub fn exit(result: i32) -> ! {
    unsafe { _exit(result); }
}

pub struct MaskSignals {
    oldmask: [u8, ..128],
}

impl MaskSignals {
    pub fn new() -> MaskSignals {
        let mut old = [0, ..128];
        let mut new = [0, ..128];
        unsafe {
            sigfillset(new.as_mut_ptr());
            sigprocmask(SIG_BLOCK, new.as_ptr(), old.as_mut_ptr());
        };
        MaskSignals {
            oldmask: old,
        }
    }
    pub fn drop(&self) {
        let mut old = [0, ..128];
        unsafe {
            sigprocmask(SIG_SETMASK, self.oldmask.as_ptr(), old.as_mut_ptr())
        };
    }
    pub fn wait(&self) -> i32 {
        let mask = [0xFF, ..128];
        let mut sig: c_int = 0;
        unsafe {
            sigwait(mask.as_ptr(), &mut sig);
        }
        return sig;
    }
}

pub fn init_prctl() {
    unsafe {
        prctl(PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0);
    }
}

pub fn get_user_name(uid: uid_t) -> Result<String, String> {
    unsafe {
        let val = getpwuid(uid);
        if val != null() {
            return Ok(from_buf((*val).pw_name as *const u8));
        }
    }
    return Err(format!("User {} not found", uid));
}
