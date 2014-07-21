use std::io;
use std::to_str::ToStr;
use std::os::{pipe, change_dir, getenv};
use std::io::{Open, Write};
use std::io::{BufferedReader, IoResult};
use std::io::fs::{mkdir, File, readlink};
use std::io::pipe::PipeStream;
use std::io::stdio::{stdout, stderr};

use collections::treemap::TreeMap;
use argparse::{ArgumentParser, Store, StoreOption, List, StoreTrue};

use super::env::{Environ, Container};
use super::linux::{make_namespace, change_root, bind_mount, mount_pseudofs};
use super::linux::{execute, forkme, wait_process};
use super::options::env_options;
use super::build::{build_container, link_container};
use libc::funcs::posix88::unistd::getuid;

static DEFAULT_PATH: &'static str =
    "/sbin:/bin:/usr/sbin:/usr/bin:/usr/local/sbin:/usr/local/bin";
static DEFAULT_SEARCH: &'static [&'static str] = &[
    "/sbin", "/bin",
    "/usr/sbin", "/usr/bin",
    "/usr/local/sbin", "/usr/local/bin",
    ];


fn mount_all(root: &Path, mount_dir: &Path, project_root: &Path)
    -> Result<(), String>
{
    let vagga_marker = mount_dir.join_many(["tmp", ".vagga"]);
    try!(bind_mount(root, mount_dir, false));
    try!(bind_mount(&Path::new("/sys"), &mount_dir.join("sys"), false));
    // TODO(tailhook) use dev in /var/lib/container-dev
    try!(bind_mount(&Path::new("/dev"), &mount_dir.join("dev"), false));
    try!(bind_mount(project_root, &mount_dir.join("work"), true));
    try!(mount_pseudofs("proc", &mount_dir.join("proc"), ""));
    // TODO(tailhook) allow customize size of tmpfs
    try!(mount_pseudofs("tmpfs", &mount_dir.join("tmp"),
                        "size=102400k,mode=1777"));
    try!(ensure_dir(&vagga_marker));
    try!(bind_mount(&vagga_marker,
                    &mount_dir.join_many(["work", ".vagga"]),
                    false));
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

fn read_env_file(path: &Path, env: &mut TreeMap<String, String>)
    -> IoResult<()>
{
    let file = try!(File::open(path).map(|f| BufferedReader::new(f)));
    let mut reader = BufferedReader::new(file);
    for (n, line) in reader.lines().enumerate() {
        let line = try!(line);
        let pair: Vec<&str> = line.as_slice().splitn('=', 1).collect();
        match pair.as_slice() {
            [key, val] => {
                let nkey = key.trim().to_string();
                if !env.contains_key(&nkey) {
                    env.insert(nkey, val.trim().to_string());
                }
            }
            _ => {
                warn!("Invalid line {} in {}",
                    n+1, path.display());
            }
        }
    }
    return Ok(());
}

fn container_environ(container: &Container, env: &mut TreeMap<String, String>)
    -> Result<(), String>
{
    for (k, v) in container.environ.iter() {
        if !env.contains_key(k) {
            env.insert(k.clone(), v.clone());
        }
    }
    match container.environ_file {
        None => {}
        Some(ref suf) => {
            let path = container.container_root.as_ref().unwrap()
                       .join(suf.as_slice().trim_left_chars('/'));
            if path.exists() {
                try!(read_env_file(&path, env).map_err(|e| format!(
                    "Error reading environment file: {}", e)));
            }
        }

    }
    return Ok(());
}


pub fn run_chroot(env: &Environ, container_root: &Path, mount_dir: &Path,
    command: String, args: Vec<String>, runenv: TreeMap<String, String>)
    -> Result<(),String>
{
    try!(mount_all(container_root, mount_dir, &env.project_root));
    debug!("Changing directory to {}", mount_dir.display());
    change_dir(mount_dir);
    try!(change_root(mount_dir));
    let reldir = match env.work_dir.path_relative_from(&env.project_root) {
        Some(path) => path,
        None => Path::new(""),
    };
    let path = Path::new("/work").join(reldir);
    debug!("Changing directory to {}", path.display());
    change_dir(&path);

    let mut runenv = runenv;
    let home = "HOME".to_string();
    if runenv.find(&home).is_none() {
        // TODO(tailhook) set home to real home if /home is mounted
        runenv.insert(home, "/homeless-shelter".to_string());
    }
    let term = "TERM".to_string();
    if runenv.find(&term).is_none() {
        runenv.insert(term, getenv("TERM").unwrap_or("linux".to_string()));
    }
    let path = "PATH".to_string();
    let mut search: Vec<&str>;
    if runenv.find(&path).is_none() {
        runenv.insert(path, DEFAULT_PATH.to_string());
        search = DEFAULT_SEARCH.to_owned();
    } else {
        search = runenv.find(&path).unwrap().as_slice().split(':').collect();
    };
    let mut environ = Vec::new();
    for (k, v) in runenv.iter() {
        environ.push(*k + "=" + *v);
    }
    try!(execute(&command,
        search.as_slice(),
        &args, &environ));
    unreachable!();
}

pub fn run_command_line(env: &mut Environ, args: Vec<String>)
    -> Result<int, String>
{
    let mut cname = "devel".to_string();
    let mut no_wrapper = false;
    let mut use_shell = false;
    let mut command: Option<String> = None;
    let mut cmdargs: Vec<String> = Vec::new();
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut cname)
            .add_argument("container", box Store::<String>,
                "A name of the container to build")
            .required();
        ap.refer(&mut command)
            .add_argument("command", box StoreOption::<String>,
                "A command to run inside container");
        ap.refer(&mut cmdargs)
            .add_argument("arguments", box List::<String>,
                "Arguments for the command")
            .required();
        ap.refer(&mut no_wrapper)
            .add_option(["-N", "--no-wrapper"], box StoreTrue,
                "Do not use `command-wrapper` configured for container");
        ap.refer(&mut use_shell)
            .add_option(["-S", "--shell"], box StoreTrue,
                "Run command with `shell` configured for container");
        env_options(env, &mut ap);
        ap.stop_on_first_argument(true);
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => return Ok(122),
        }
    }
    let container = try!(env.get_container(&cname));

    if command.is_none() {
        if container.default_command.is_some() {
            let cmd = container.default_command.as_ref().unwrap();
            cmdargs.push_all(cmd.as_slice());
        } else {
            return Err(format!("No command specified and no default command"));
        }
    } else {
        cmdargs.insert(0, command.unwrap());
        if use_shell {
            cmdargs = container.shell.clone() + cmdargs;
        } else if !no_wrapper && container.command_wrapper.is_some() {
            cmdargs = container.command_wrapper.as_ref().unwrap().clone()+cmdargs;
        }
    }

    let cmd = cmdargs.shift().unwrap();
    return internal_run(env, container, cmd, cmdargs, TreeMap::new());
}

pub fn internal_run(env: &Environ, container_: Container,
    command: String, cmdargs: Vec<String>, runenv: TreeMap<String, String>)
    -> Result<int, String>
{
    let mut container = container_;
    if env.settings.version_check {
        try!(build_container(env, &mut container, false));
        try!(link_container(env, &container));
    } else {
        let lnk = env.local_vagga.join(container.fullname.as_slice());
        container.container_root = match readlink(&lnk) {
            Ok(path) => Some(lnk.dir_path().join(path)),
            Err(e) => return Err(format!("Container {} not found: {}",
                                         container.fullname, e)),
        };
    };
    let mut runenv = runenv;
    try!(container_environ(&container, &mut runenv));
    let container_root = container.container_root.as_ref().unwrap();
    info!("Running {}: {} {}", container_root.display(), command, cmdargs);

    let uid = unsafe { getuid() };

    for dir in ["proc", "sys", "dev", "work", "tmp"].iter() {
        try!(ensure_dir(&container_root.join(*dir)));
    }

    let mount_dir = env.project_root.join_many([".vagga", ".mnt"]);
    try!(ensure_dir(&mount_dir));
    try!(make_namespace());

    let mut pipe = match PipeStream::pair() {
        Ok(pipe) => pipe,
        Err(x) => return Err(format!("Can't create pipe: {}", x)),
    };

    let pid = try!(forkme());
    if pid == 0 {
        let mut buf: [u8,..1] = [0];
        match pipe.reader.read(buf) {
            Ok(_) => {}
            Err(x) => return Err(format!("Can't read from pipe: {}", x)),
        }
        drop(pipe);
        try!(run_chroot(env, container_root, &mount_dir,
            command, cmdargs, runenv));
        unreachable!();
    } else {
        // TODO(tailhook) set uid map from config
        let uid_map = format!("0 {} 1", uid);
        debug!("Writing uid_map: {}", uid_map);
        match File::open_mode(&Path::new("/proc")
                          .join(pid.to_str())
                          .join("uid_map"), Open, Write)
                .write_str(uid_map.as_slice()) {
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

        return wait_process(pid);
    }
}
