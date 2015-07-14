#![allow(dead_code)]

use std::env::current_dir;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::ffi::{CString};
use std::fmt::{Debug, Formatter};
use std::fmt::Error as FormatError;
use std::io::Error as IoError;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::ptr::null;

use super::pipe::CPipe;
use super::uidmap::{Uidmap, get_max_uidmap, apply_uidmap};

use libc::{c_int, c_char, pid_t};
use self::Namespace::*;


#[derive(Debug, Hash)]
pub enum Namespace {
    NewMount,
    NewUts,
    NewIpc,
    NewUser,
    NewPid,
    NewNet,
}

pub struct Command {
    pub name: String,
    chroot: CString,
    executable: CString,
    arguments: Vec<CString>,
    environment: BTreeMap<String, String>,
    namespaces: HashSet<Namespace>,
    restore_sigmask: bool,
    user_id: i32,
    workdir: CString,
    uidmap: Option<Uidmap>,
    stdin: i32,
    stdout: i32,
    stderr: i32,
}

impl Debug for Command {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FormatError> {
        write!(fmt, "{:?} {:?}", self.executable, self.arguments)
    }
}

impl Command {
    pub fn new(name: String, cmd: &Path) -> Command {
        let slc = cmd.as_os_str().as_bytes();
        return Command {
            name: name,
            chroot: CString::new("/").unwrap(),
            workdir: CString::new(current_dir().unwrap()
                .as_os_str().as_bytes().to_vec()).unwrap(),
            executable: CString::new(slc.to_vec()).unwrap(),
            arguments: vec!(CString::new(slc.to_vec()).unwrap()),
            namespaces: HashSet::new(),
            environment: BTreeMap::new(),
            restore_sigmask: true,
            user_id: 0,
            uidmap: None,
            stdin: 0,
            stdout: 1,
            stderr: 2,
        };
    }
    pub fn set_user_id(&mut self, uid: i32) {
        self.user_id = uid;
    }
    pub fn set_stdout_fd(&mut self, fd: i32) {
        self.stdout = fd;
    }
    pub fn set_stderr_fd(&mut self, fd: i32) {
        self.stderr = fd;
    }
    pub fn set_stdin_fd(&mut self, fd: i32) {
        self.stdin = fd;
    }
    pub fn chroot(&mut self, dir: &Path) {
        self.chroot = CString::from_slice(dir.container_as_bytes());
    }
    pub fn set_workdir(&mut self, dir: &Path) {
        self.workdir = CString::from_slice(dir.container_as_bytes());
    }
    pub fn keep_sigmask(&mut self) {
        self.restore_sigmask = false;
    }
    pub fn arg<T:AsRef<[u8]>>(&mut self, arg: T) {
        self.arguments.push(CString::from_slice(arg.as_ref()));
    }
    pub fn args<T:AsRef<[u8]>>(&mut self, arg: &[T]) {
        self.arguments.extend(arg.iter()
            .map(|v| CString::from_slice(v.as_ref())));
    }
    pub fn set_env(&mut self, key: String, value: String)
    {
        self.environment.insert(key, value);
    }

    pub fn update_env<'x, I: Iterator<Item=(String, String)>>(&mut self,
        mut env: I)
    {
        for (k, v) in env {
            self.environment.insert(k, v);
        }
    }
    pub fn set_max_uidmap(&mut self) {
        self.namespaces.insert(NewUser);
        self.uidmap = Some(get_max_uidmap().unwrap());
    }
    pub fn set_uidmap(&mut self, uidmap: Uidmap) {
        self.namespaces.insert(NewUser);
        self.uidmap = Some(uidmap);
    }
    pub fn network_ns(&mut self) {
        self.namespaces.insert(NewNet);
        self.namespaces.insert(NewUts);
    }
    pub fn container(&mut self) {
        // Network and user namespaces are set separately
        self.namespaces.insert(NewMount);
        self.namespaces.insert(NewIpc);
        self.namespaces.insert(NewPid);
    }
    pub fn spawn(&self) -> Result<pid_t, String> {
        let mut exec_args: Vec<*const u8> = self.arguments.iter()
            .map(|a| a.as_bytes().as_ptr()).collect();
        exec_args.push(null());
        let environ_cstr: Vec<CString> = self.environment.iter()
            .map(|(k, v)| CString::from_slice(
                (k.clone() + "=" + &v).as_bytes()))
            .collect();
        let mut exec_environ: Vec<*const u8> = environ_cstr.iter()
            .map(|p| p.as_bytes().as_ptr()).collect();
        exec_environ.push(null());

        let logprefix = CString::from_slice(format!(
            // Only errors are logged from C code
            "ERROR:lithos::container.c: [{}]", self.name
            ).as_bytes());

        let pipe = try!(CPipe::new()
                        .map_err(|e| format!("Error creating pipe: {}", e)));
        let pid = unsafe { execute_command(&CCommand {
            pipe_reader: pipe.reader_fd(),
            logprefix: logprefix.as_bytes().as_ptr(),
            fs_root: self.chroot.as_bytes().as_ptr(),
            exec_path: self.executable.as_bytes().as_ptr(),
            exec_args: exec_args[..].as_ptr(),
            exec_environ: exec_environ[..].as_ptr(),
            namespaces: convert_namespaces(self.namespaces),
            user_id: self.user_id,
            restore_sigmask: if self.restore_sigmask { 1 } else { 0 },
            workdir: self.workdir.as_ptr(),
            stdin: self.stdin,
            stdout: self.stdout,
            stderr: self.stderr,
        }) };
        if pid < 0 {
            return Err(format!("Error executing: {}", IoError::last_error()));
        }
        if let Some(uidmap) = self.uidmap.as_ref() {
            try!(apply_uidmap(pid, uidmap)
                .map_err(|e| format!("Error writing uid_map: {}", e)));
        }
        try!(pipe.wakeup()
            .map_err(|e| format!("Error waking up process: {}. \
                Probably child already dead", e)));
        return Ok(pid)
    }
}

pub fn convert_namespace(value: Namespace) -> c_int {
    match value {
        NewMount => CLONE_NEWNS,
        NewUts => CLONE_NEWUTS,
        NewIpc => CLONE_NEWIPC,
        NewUser => CLONE_NEWUSER,
        NewPid => CLONE_NEWPID,
        NewNet => CLONE_NEWNET,
    }
}


fn convert_namespaces(set: HashSet<Namespace>) -> c_int {
    let mut ns = 0;
    for i in set.iter() {
        ns |= convert_namespace(i);
    }
    return ns;
}

static CLONE_NEWNS: c_int = 0x00020000;   /* Set to create new namespace.  */
static CLONE_NEWUTS: c_int = 0x04000000;  /* New utsname group.  */
static CLONE_NEWIPC: c_int = 0x08000000;  /* New ipcs.  */
static CLONE_NEWUSER: c_int = 0x10000000; /* New user namespace.  */
static CLONE_NEWPID: c_int = 0x20000000;  /* New pid namespace.  */
static CLONE_NEWNET: c_int = 0x40000000;  /* New network namespace.  */

#[repr(C)]
pub struct CCommand {
    namespaces: c_int,
    pipe_reader: c_int,
    user_id: c_int,
    restore_sigmask: c_int,
    stdin: c_int,
    stdout: c_int,
    stderr: c_int,
    logprefix: *const u8,
    fs_root: *const u8,
    exec_path: *const u8,
    exec_args: *const*const u8,
    exec_environ: *const*const u8,
    workdir: *const c_char,
}

#[link(name="container", kind="static")]
extern {
    fn execute_command(cmd: *const CCommand) -> pid_t;
}
