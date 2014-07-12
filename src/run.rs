use std::io;
use std::to_str::ToStr;
use std::os::{pipe, change_dir, getenv};
use std::io::{Open, Write};
use std::io::fs::{mkdir, File};
use std::io::pipe::PipeStream;
use std::io::stdio::{stdout, stderr};

use argparse::{ArgumentParser, Store, StoreOption, List};

use super::env::Environ;
use super::config::Config;
use super::linux::{make_namespace, change_root, bind_mount, mount_pseudofs};
use super::linux::{execute, forkme, wait_process};
use libc::funcs::posix88::unistd::getuid;


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


pub fn run_chroot(env: &Environ, container_root: &Path, mount_dir: &Path,
    command: &String, args: &Vec<String>)
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

    // TODO(tailhook) set environment from config
    let environ = vec!(
        "PATH=/bin:/usr/bin:/usr/local/bin".to_string(),
        "HOME=/non-existent".to_string(),
        "TERM=".to_string() + getenv("TERM").unwrap_or("linux".to_string()),
        );
    try!(execute(command, ["/bin", "/usr/bin", "/usr/local/bin"],
        args, &environ));
    unreachable!();
}

pub fn run_user_command(env: &Environ, config: &Config,
    cmdname: &String, args: Vec<String>)
    -> Result<int, String>
{
    let command = match config.commands.find(cmdname) {
        Some(c) => c,
        None => {
            return Err(format!("Can't find command {} in config",
                               cmdname));
        }
    };
    let cname = match command.container {
        Some(ref name) => name.clone(),
        None => unimplemented!(),
    };
    let container = match config.containers.find(&cname) {
        Some(c) => c,
        None => {
            return Err(format!("Can't find container {} for command {}",
                               command.container, cmdname));
        }
    };
    match (&container.wrapper_script, &command.run) {
        (&Some(ref wrapper), &Some(ref cmdline)) =>
            return _run(env, &cname, wrapper,
                &(vec!("/bin/sh".to_string(), "-c".to_string(),
                       cmdline.clone()) + args.slice_from(1))),
        (&None, &Some(ref cmdline)) =>
            return _run(env, &cname, &"/bin/sh".to_string(),
                &(vec!("-c".to_string(), cmdline.clone()) + args.slice_from(1))),
        (_, &None) => unimplemented!(),
    }
}

pub fn run_command(env: &Environ, config: &Config, args: Vec<String>)
    -> Result<int, String>
{
    let mut cname = "devel".to_string();
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
        ap.stop_on_first_argument(true);
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
    match (&command, &container.default_command, &container.wrapper_script) {
        (&None, &None, _) =>
            return Err(format!("No command specified and no default command")),
        (&None, &Some(ref cmd), &Some(ref wrapper)) => {
            cmdargs.insert(0, cmd.clone());
            return _run(env, &cname, wrapper, &cmdargs);
        }
        (&Some(ref cmd), _, &Some(ref wrapper)) => {
            cmdargs.insert(0, cmd.clone());
            return _run(env, &cname, wrapper, &cmdargs);
        }
        (&None, &Some(ref cmd), &None) =>
            return _run(env, &cname, cmd, &cmdargs),
        (&Some(ref cmd), _, &None) =>
            return _run(env, &cname, cmd, &cmdargs),
    }
}

pub fn _run(env: &Environ, container: &String,
    command: &String, cmdargs: &Vec<String>)
    -> Result<int, String>
{
    info!("Running {}: {} {}", container, command, cmdargs);
    let container_dir = env.project_root.join_many(
        [".vagga", container.as_slice()]);
    let container_root = container_dir.join("root");
    let uid = unsafe { getuid() };

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

    let pid = try!(forkme());
    if pid == 0 {
        let mut buf: [u8,..1] = [0];
        match pipe.reader.read(buf) {
            Ok(_) => {}
            Err(x) => return Err(format!("Can't read from pipe: {}", x)),
        }
        drop(pipe);
        try!(run_chroot(env, &container_root, &mount_dir, command, cmdargs));
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
