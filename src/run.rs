use std::io;
use std::to_str::ToStr;
use std::os::pipe;
use std::io::{Open, Write};
use std::io::fs::{mkdir, File};
use std::io::pipe::PipeStream;
use std::io::stdio::{stdout, stderr};

use argparse::{ArgumentParser, Store, List};

use super::env::Environ;
use super::config::Config;
use super::linux::{make_namespace, change_root, bind_mount, mount_pseudofs};
use super::linux::{execute, forkme, wait_process};


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
    try!(change_root(mount_dir));
    // TODO(tailhook) set environment from config
    let environ = vec!("PATH=/bin:/usr/bin:/usr/local/bin".to_string());
    try!(execute(command, args, &environ));
    unreachable!();
}

pub fn run_user_command(env: &Environ, config: &Config,
    cmdname: &String, args: Vec<String>)
    -> Result<int, String> {
    unimplemented!();
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
    match config.containers.find(&cname) {
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

    let pid = try!(forkme());
    if pid == 0 {
        let mut buf: [u8,..1] = [0];
        match pipe.reader.read(buf) {
            Ok(_) => {}
            Err(x) => return Err(format!("Can't read from pipe: {}", x)),
        }
        drop(pipe);
        try!(run_chroot(env, &container_root, &mount_dir, &command, &cmdargs));
        unreachable!();
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

        return wait_process(pid);
    }
}
