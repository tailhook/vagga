use std::rc::Rc;
use std::os::{Pipe, pipe};
use std::io::PipeStream;
use std::io::ALL_PERMISSIONS;
use std::os::{getcwd, set_exit_status, self_exe_path, getenv};
use std::cell::RefCell;
use std::io::fs::{rmdir_recursive, mkdir_recursive, mkdir, rename};
use std::io::fs::PathExtensions;
use libc::funcs::posix88::unistd::close;

use container::mount::{bind_mount};
use container::monitor::{Monitor, Executor, MonitorStatus, Shutdown};
use container::monitor::{Killed, Exit};
use container::container::{Command};


struct RunBuilder {
    container: String,
}

struct RunVersion {
    container: String,
    pipe: Pipe,
    result: Rc<RefCell<String>>,
}

impl Executor for RunVersion {
    fn command(&self) -> Command {
        let mut cmd = Command::new("vagga_version".to_string(),
            Path::new("/vagga/bin/vagga_version"));
        cmd.keep_sigmask();
        cmd.arg(self.container.as_slice());
        cmd.set_env("TERM".to_string(), "dumb".to_string());
        cmd.set_stdout_fd(self.pipe.writer);
        if let Some(x) = getenv("RUST_LOG") {
            cmd.set_env("RUST_LOG".to_string(), x);
        }
        if let Some(x) = getenv("RUST_BACKTRACE") {
            cmd.set_env("RUST_BACKTRACE".to_string(), x);
        }
        return cmd;
    }
    fn finish(&self, status: int) -> MonitorStatus {
        unsafe { close(self.pipe.writer) };
        let mut rd = PipeStream::open(self.pipe.reader);
        *self.result.borrow_mut() = rd.read_to_string()
                                      .unwrap_or("".to_string());
        return Shutdown(status)
    }
}

impl Executor for RunBuilder {
    fn command(&self) -> Command {
        let mut cmd = Command::new("vagga_build".to_string(),
            Path::new("/vagga/bin/vagga_build"));
        cmd.keep_sigmask();
        cmd.arg(self.container.as_slice());
        cmd.set_env("TERM".to_string(),
                    getenv("TERM").unwrap_or("dumb".to_string()));
        if let Some(x) = getenv("RUST_LOG") {
            cmd.set_env("RUST_LOG".to_string(), x);
        }
        if let Some(x) = getenv("RUST_BACKTRACE") {
            cmd.set_env("RUST_BACKTRACE".to_string(), x);
        }
        return cmd;
    }
    fn finish(&self, status: int) -> MonitorStatus {
        return Shutdown(status)
    }
}

pub fn prepare_tmp_root_dir(path: &Path) -> Result<(), String> {
    if path.exists() {
        try!(rmdir_recursive(path)
             .map_err(|x| format!("Error creating directory: {}", x)));
    }
    try!(mkdir_recursive(path, ALL_PERMISSIONS)
         .map_err(|x| format!("Error creating directory: {}", x)));
    let rootdir = path.join("root");
    try!(mkdir(&rootdir, ALL_PERMISSIONS)
         .map_err(|x| format!("Error creating directory: {}", x)));
    let tgtroot = Path::new("/vagga/root");
    try!(mkdir(&tgtroot, ALL_PERMISSIONS)
         .map_err(|x| format!("Error creating directory: {}", x)));
    try!(bind_mount(&rootdir, &tgtroot));
    return Ok(());
}

pub fn commit_root(tmp_path: &Path, final_path: &Path) -> Result<(), String> {
    let mut path_to_remove = None;
    if final_path.exists() {
        let rempath = tmp_path.with_filename(
            tmp_path.filename_str().unwrap().to_string() + ".old");
        try!(rename(final_path, &rempath)
             .map_err(|x| format!("Error renaming old dir: {}", x)));
        path_to_remove = Some(rempath);
    }
    try!(rename(tmp_path, final_path)
         .map_err(|x| format!("Error renaming dir: {}", x)));
    if let Some(ref path_to_remove) = path_to_remove {
        try!(rmdir_recursive(path_to_remove)
             .map_err(|x| format!("Error removing old dir: {}", x)));
    }
    return Ok(());
}

pub fn print_version_hash(container: String) -> Result<(), int> {
    let mut mon = Monitor::new();
    let ver = Rc::new(RefCell::new("".to_string()));
    mon.add(Rc::new("version".to_string()), box RunVersion {
        container: container,
        pipe: unsafe { pipe() }.ok().expect("Can't create pipe"),
        result: ver.clone(),
    });
    match mon.run() {
        Killed => return Err(1),
        Exit(0) => {},
        Exit(29) => return Err(29),
        Exit(val) => return Err(val),
    };
    println!("{}", ver.borrow());
    return Ok(());
}

pub fn build_container(container: String) -> Result<String, int> {
    let mut mon = Monitor::new();
    let ver = Rc::new(RefCell::new("".to_string()));
    mon.add(Rc::new("version".to_string()), box RunVersion {
        container: container.clone(),
        pipe: unsafe { pipe() }.ok().expect("Can't create pipe"),
        result: ver.clone(),
    });
    match mon.run() {
        Killed => return Err(1),
        Exit(0) => {
            debug!("Container version: {}", ver.borrow());
            let name = format!("{}.{}", container,
                ver.borrow().as_slice().slice_to(8));
            let finalpath = Path::new("/vagga/roots")
                .join(name.as_slice());
            if finalpath.exists() {
                debug!("Path {} is already built",
                       finalpath.display());
                return Ok(name);
            }
        },
        Exit(29) => {},
        Exit(val) => return Err(val),
    };
    debug!("Container version: {}", ver.borrow());
    let tmppath = Path::new(format!("/vagga/roots/.tmp.{}", container));
    match prepare_tmp_root_dir(&tmppath) {
        Ok(()) => {}
        Err(x) => {
            error!("Error preparing root dir: {}", x);
            return Err(124);
        }
    }
    mon.add(Rc::new("build".to_string()), box RunBuilder {
        container: container.to_string(),
    });
    match mon.run() {
        Killed => return Err(1),
        Exit(0) => {},
        Exit(val) => return Err(val),
    };
    if ver.borrow().len() != 64 {
        mon.add(Rc::new("version".to_string()), box RunVersion {
            container: container.to_string(),
            pipe: unsafe { pipe() }.ok().expect("Can't create pipe"),
            result: ver.clone(),
        });
        match mon.run() {
            Killed => return Err(1),
            Exit(0) => {},
            Exit(29) => {
                error!("Internal Error: \
                        Can't version even after build");
                return Err(124);
            },
            Exit(val) => return Err(val),
        };
    }
    let name = format!("{}.{}", container,
        ver.borrow().as_slice().slice_to(8));
    let finalpath = Path::new("/vagga/roots").join(name.as_slice());
    debug!("Committing {} -> {}", tmppath.display(),
                                  finalpath.display());
    match commit_root(&tmppath, &finalpath) {
        Ok(()) => {}
        Err(x) => {
            error!("Error committing root dir: {}", x);
            return Err(124);
        }
    }
    return Ok(name);
}
