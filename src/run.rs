use std::to_str::ToStr;
use std::os::getenv;
use std::io::{Open, Write};
use std::io::{BufferedReader, IoResult};
use std::io::fs::File;
use std::io::stdio::{stdout, stderr};

use libc::pid_t;
use collections::treemap::TreeMap;
use argparse::{ArgumentParser, Store, StoreOption, List, StoreTrue};

use super::env::{Environ, Container};
use super::linux::{wait_process, ensure_dir, run_container, CPipe};
use super::options::env_options;
use super::build::ensure_container;
use libc::funcs::posix88::unistd::getuid;


static DEFAULT_PATH: &'static str =
    "/sbin:/bin:/usr/sbin:/usr/bin:/usr/local/sbin:/usr/local/bin";


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
    let mut container = try!(env.get_container(&cname));

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
    try!(ensure_container(env, &mut container));
    let pid = try!(internal_run(env, &container,
        cmd, cmdargs, TreeMap::new()));
    return wait_process(pid);
}

pub fn internal_run(env: &Environ, container: &Container,
    command: String, cmdargs: Vec<String>, runenv: TreeMap<String, String>)
    -> Result<pid_t, String>
{
    let mut runenv = runenv;
    try!(container_environ(container, &mut runenv));
    let container_root = container.container_root.as_ref().unwrap();
    info!("Running {}: {} {}", container_root.display(), command, cmdargs);

    let uid = unsafe { getuid() };

    let mount_dir = env.project_root.join_many([".vagga", ".mnt"]);
    try!(ensure_dir(&mount_dir));

    let pipe = try!(CPipe::new());

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
    if runenv.find(&path).is_none() {
        runenv.insert(path, DEFAULT_PATH.to_string());
    }

    let pid = try!(run_container(&pipe, env, container,
        &command, cmdargs.as_slice(), &runenv));

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

    try!(pipe.wakeup());
    return Ok(pid);
}
