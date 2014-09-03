use std::os::getenv;
use std::io::{BufferedReader, IoResult, AllPermissions};
use std::io::fs::{File, copy, rename, mkdir};
use std::io::stdio::{stdout, stderr};
use std::default::Default;

use libc::pid_t;
use collections::treemap::TreeMap;
use argparse::{ArgumentParser, Store, StoreOption, List, StoreTrue, StoreConst};
use argparse::{StoreFalse};

use super::uidmap::write_uid_map;
use super::monitor::Monitor;
use super::env::{Environ, Container};
use super::linux::{ensure_dir, RunOptions, run_container, CPipe};
use super::linux::{BindRO, BindROTmp};
use super::utils::run::{container_volumes};
use super::options::env_options;
use super::build::ensure_container;

use Pid1 = super::linux;


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
    let mut resolv: bool = true;
    let mut ropts: RunOptions = Default::default();
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut cname)
            .add_argument("container", box Store::<String>,
                "A name of the container to build")
            .required();
        ap.refer(&mut command)
            .add_argument("command", box StoreOption::<String>,
                "A command to run inside container")
            .required();
        ap.refer(&mut cmdargs)
            .add_argument("arguments", box List::<String>,
                "Arguments for the command");
        ap.refer(&mut no_wrapper)
            .add_option(["-N", "--no-wrapper"], box StoreTrue,
                "Do not use `command-wrapper` configured for container");
        ap.refer(&mut use_shell)
            .add_option(["-S", "--shell"], box StoreTrue,
                "Run command with `shell` configured for container");
        ap.refer(&mut use_shell)
            .add_option(["-S", "--shell"], box StoreTrue,
                "Run command with `shell` configured for container");
        ap.refer(&mut resolv)
            .add_option(["--no-resolv"], box StoreFalse,
                "Do not copy /etc/resolv.conf");
        ap.refer(&mut ropts.pid1mode)
            .add_option(["--wait"], box StoreConst(Pid1::Wait),
                "Spawn a supervisor as pid 1 and wait until target command \
                 exits. But if it spawned any children, they will be killed \
                 at the death of the target command (default)")
            .add_option(["--wait-all-children"],
                box StoreConst(Pid1::WaitAllChildren),
                "Spawn a supervisor as pid 1 and wait until target command \
                 and all of its children are dead. (Dangerous in some cases \
                 as some non useful child process may block container \
                 shutdown).")
            .add_option(["--exec"], box StoreConst(Pid1::Exec),
                "Execute the target command as pid 1. Should not be used \
                 unless the target command is a process manager itself and/or \
                 supposed to work as pid 1.");
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

    ropts.mounts = container_volumes(env, &container);
    let mut monitor = Monitor::new(true);
    let pid = try!(internal_run(env, &container, ropts, resolv,
        &env.work_dir, cmd, cmdargs, TreeMap::new()));
    monitor.add("child".to_string(), pid);
    monitor.wait_all();
    return Ok(monitor.get_status());
}

pub fn internal_run(env: &Environ, container: &Container,
    ropts: RunOptions, resolv: bool, work_dir: &Path,
    command: String, cmdargs: Vec<String>, runenv: TreeMap<String, String>)
    -> Result<pid_t, String>
{
    let mut runenv = runenv;
    env.populate_environ(&mut runenv);
    try!(container_environ(container, &mut runenv));
    let container_root = container.container_root.as_ref().unwrap();
    info!("Running {}: {} {}", container_root.display(), command, cmdargs);

    let mount_dir = env.project_root.join_many([".vagga", ".mnt"]);
    try!(ensure_dir(&mount_dir));

    if resolv {
        let etc = container_root.join("etc");
        if !etc.exists() {
            try!(mkdir(&etc, AllPermissions)
                .map_err(|e| format!("Error creating /etc: {}", e)));
        }
        try!(copy(&Path::new("/etc/resolv.conf"),
                  &etc.join("resolv.conf.tmp"))
            .map_err(|e| format!("Error copying resolv.conf: {}", e)));
        try!(rename(&etc.join("resolv.conf.tmp"),
                    &etc.join("resolv.conf"),)
            .map_err(|e| format!("Error copying resolv.conf: {}", e)));
    }


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

    let mut ropts = ropts;
    let mount_dir = env.local_vagga.join(".mnt");
    ropts.mounts.push(
        BindRO(env.vagga_inventory.join("markerdir").to_c_str(),
               mount_dir.join("work").join(".vagga").to_c_str()));
    ropts.mounts.push(BindROTmp(env.vagga_inventory.to_c_str(),
                mount_dir.join_many(["tmp", "inventory"]).to_c_str()));

    let pipe = try!(CPipe::new());
    let pid = try!(run_container(&pipe, env,
        container.container_root.as_ref().unwrap(),
        &ropts, work_dir, &command, cmdargs.as_slice(), &runenv));

    try!(write_uid_map(pid, &container.uids, &container.gids));

    try!(pipe.wakeup());
    return Ok(pid);
}
