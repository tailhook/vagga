use std::io;
use std::io::timer::sleep;
use std::os::getenv;
use std::os::{errno, error_string};
use std::c_str::CString;
use std::to_str::ToStr;
use std::ptr::null;
use std::os::pipe;
use std::io::{Open, Write};
use std::io::fs::{mkdir, rmdir_recursive, rename, File};
use std::io::process::{ExitStatus, ExitSignal, Command, Ignored, InheritFd};
use std::io::pipe::PipeStream;
use std::io::stdio::{stdout, stderr};
use libc::{c_int, c_char, c_ulong, c_void, pid_t};
use libc::funcs::posix88::unistd::{fork, usleep, write, getuid};

use argparse::{ArgumentParser, Store, List};

use super::env::Environ;
use super::config::Config;


// errno.h
static EINTR: int = 4;

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

fn make_namespace() -> Result<(), String> {
    let rc = unsafe {
        unshare(CLONE_NEWNS|CLONE_NEWIPC|CLONE_NEWUSER|CLONE_NEWPID)
    };
    if rc != 0 {
        return Err(format!("Error making namespace: {}",
            error_string(errno() as uint)));
    }
    return Ok(());
}

fn change_root(root: &Path) -> Result<(), String> {
    let rc = unsafe { chroot(root.to_c_str().unwrap()) };
    if rc != 0 {
        return Err(format!("Error changing root: {}",
            error_string(errno() as uint)));
    }
    return Ok(());
}

fn mount_all(root: &Path, mount_dir: &Path, project_root: &Path)
    -> Result<(), String>
{
    let vagga_marker = mount_dir.join_many(["tmp", ".vagga"]);
    unsafe {
        if mount(root.to_c_str().unwrap(), mount_dir.to_c_str().unwrap(),
            null(), MS_BIND|MS_REC|MS_RDONLY, null()) != 0 {
            return Err(format!("Error mounting root: {}",
                error_string(errno() as uint)));
        }
        if mount("/sys".to_c_str().unwrap(),
                 mount_dir.join("sys").to_c_str().unwrap(),
                 null(), MS_BIND|MS_REC|MS_RDONLY, null()) != 0 {
            return Err(format!("Error mounting /sys: {}",
                error_string(errno() as uint)));
        }
        // TODO(tailhook) use dev in /var/lib/container-dev
        if mount("/dev".to_c_str().unwrap(),
                 mount_dir.join("dev").to_c_str().unwrap(),
                 null(), MS_BIND|MS_REC|MS_RDONLY, null()) != 0 {
            return Err(format!("Error mounting /dev: {}",
                error_string(errno() as uint)));
        }
        if mount(project_root.to_c_str().unwrap(),
                 mount_dir.join("work").to_c_str().unwrap(),
                 null(), MS_BIND|MS_REC, null()) != 0 {
            return Err(format!("Error mounting /work: {}",
                error_string(errno() as uint)));
        }
        if mount("proc".to_c_str().unwrap(),
                 mount_dir.join("proc").to_c_str().unwrap(),
                 "proc".to_c_str().unwrap(), 0, null()) != 0 {
            return Err(format!("Error mounting /proc: {}",
                error_string(errno() as uint)));
        }
        if mount("tmpfs".to_c_str().unwrap(),
                 mount_dir.join("tmp").to_c_str().unwrap(),
                 "tmpfs".to_c_str().unwrap(),
                 MS_NODEV|MS_NOSUID,
                 "size=102400k,mode=1777".to_c_str().unwrap()) != 0 {
            return Err(format!("Error mounting /proc: {}",
                error_string(errno() as uint)));
        }
        try!(ensure_dir(&vagga_marker));
        if mount(vagga_marker.to_c_str().unwrap(),
                 mount_dir.join_many(["work", ".vagga"]).to_c_str().unwrap(),
                 null(), MS_BIND|MS_REC, null()) != 0 {
            return Err(format!("Error mounting /work/.vagga: {}",
                error_string(errno() as uint)));
        }
    }
    match File::open_mode(&vagga_marker.join("CONTAINED.txt"), Open, Write)
        .write_line("You are running in vagga container.") {
        Ok(()) => {}
        Err(e) => return Err(format!("Can't write CONTAINED.txt: {}", e)),
    }
    return Ok(());
}

fn ensure_dir(p: &Path) -> Result<(),String> {
    if p.exists() {
        return Ok(());
    }
    return mkdir(p, io::UserRWX).map_err(|e| { e.to_str() });
}


pub fn run_chroot(env: &Environ, container_root: &Path, mount_dir: &Path,
    command: &String, args: &Vec<String>)
    -> Result<int,String>
{
    try!(mount_all(container_root, mount_dir, &env.project_root));
    try!(change_root(mount_dir));

    let mut args: Vec<CString> = args.iter()
        .map(|s| { s.to_c_str() }).collect();
    args.insert(0, command.to_c_str());
    unsafe {
        let mut argv: Vec<*c_char> =
            args.move_iter().map(|s| { s.unwrap() }).collect();
        argv.push(null());
        // TODO(tailhook) set environment from config
        let envp = vec!(
            "PATH=/bin:/usr/bin:/usr/local/bin".to_c_str().unwrap(),
            null());
        // TODO(tailhook) chdir
        execve(
            command.to_c_str().unwrap(),
            argv.as_ptr(),
            envp.as_ptr(),
            );
    }
    return Err(format!("Error executing command {}: {}",
        command, error_string(errno() as uint)));
}

pub fn run_command(env: &Environ, config: &Config, args: Vec<String>)
    -> Result<int, String>
{
    let mut cname = "devel".to_string();
    let mut command = "".to_string();
    let mut cmdargs = Vec::new();
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut cname)
            .add_argument("container", box Store::<String>,
                "A name of the container to build")
            .required();
        ap.refer(&mut command)
            .add_argument("command", box Store::<String>,
                "A command to run inside container")
            .required();
        ap.refer(&mut cmdargs)
            .add_argument("arguments", box List::<String>,
                "Arguments for the command")
            .required();
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => return Ok(122),
        }
    }
    let container = match config.containers.find(&cname) {
        Some(c) => c,
        None => {
            return Err(format!("Can't find container {} in config",
                               cname));
        }
    };
    info!("Running {}: {} {}", cname, command, cmdargs);

    let container_dir = env.project_root.join_many(
        [".vagga", cname.as_slice()]);
    let container_root = container_dir.join("root");

    for dir in ["proc", "sys", "dev", "work", "tmp"].iter() {
        try!(ensure_dir(&container_root.join(*dir)));
    }

    let mount_dir = env.project_root.join_many([".vagga", "mnt"]);
    try!(ensure_dir(&mount_dir));
    try!(make_namespace());

    let mut pipe = match PipeStream::pair() {
        Ok(pipe) => pipe,
        Err(x) => return Err(format!("Can't create pipe: {}", x)),
    };

    let mut pid = unsafe { fork() };
    if(pid == 0) {
        let mut buf: [u8,..1] = [0];
        match pipe.reader.read(buf) {
            Ok(_) => {}
            Err(x) => return Err(format!("Can't read from pipe: {}", x)),
        }
        drop(pipe);
        return run_chroot(env, &container_root, &mount_dir, &command, &cmdargs);
    } else {
        match File::open_mode(&Path::new("/proc")
                          .join(pid.to_str())
                          .join("uid_map"), Open, Write)
                .write_str("0 1000 1") {
            Ok(()) => {}
            Err(e) => return Err(format!(
                "Error writing uid mapping: {}", e)),
        }

        match pipe.writer.write_char('x') {
            Ok(_) => {}
            Err(e) => return Err(format!(
                "Error writing to pipe: {}", e)),
        }
        drop(pipe);

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
            return Ok(status as int);
        }
    }
}
