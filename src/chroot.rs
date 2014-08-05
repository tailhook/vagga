use std::os;
use std::io::{Open, Write};
use std::io::stdio::{stdout, stderr};
use std::io::fs::{File, rename, copy};
use std::default::Default;

use argparse::{ArgumentParser, Store, List, StoreTrue, StoreFalse};
use collections::treemap::TreeMap;

use super::monitor::Monitor;
use super::env::Environ;
use super::linux::{ensure_dir, RunOptions, run_container, CPipe};
use super::options::env_options;
use libc::funcs::posix88::unistd::getuid;


pub fn run_chroot(env: &mut Environ, args: Vec<String>)
    -> Result<int, String>
{
    let mut root: Path = Path::new("");
    let mut command: String = "".to_string();
    let mut cmdargs: Vec<String> = Vec::new();
    let mut ropts: RunOptions = Default::default();
    let mut resolv: bool = true;
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut root)
            .add_argument("newroot", box Store::<Path>,
                "The new root directory")
            .required();
        ap.refer(&mut command)
            .add_argument("command", box Store::<String>,
                "A command to run inside container")
            .required();
        ap.refer(&mut cmdargs)
            .add_argument("arguments", box List::<String>,
                "Arguments for the command");
        ap.refer(&mut ropts.writeable)
            .add_option(["--writeable"], box StoreTrue,
                "Mount container as writeable. Useful mostly in scripts \
                 building containers itself");
        ap.refer(&mut ropts.inventory)
            .add_option(["--inventory"], box StoreTrue,
                "Mount inventory folder of vagga inside container \
                 /tmp/vagga-inventory");
        ap.refer(&mut resolv)
            .add_option(["--no-resolv"], box StoreFalse,
                "Do not copy /etc/resolv.conf");
        env_options(env, &mut ap);
        ap.stop_on_first_argument(true);
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => return Ok(122),
        }
    }
    if !env.project_root.is_ancestor_of(&root) {
        return Err(format!("Trying to chroot into wrong folder: {}",
            root.display()));
    }

    for dir in ["proc", "sys", "dev", "work", "tmp"].iter() {
        try!(ensure_dir(&root.join(*dir)));
    }
    if resolv {
        try!(ensure_dir(&root.join("etc")));
        try!(copy(&Path::new("/etc/resolv.conf"),
                  &root.join("etc/resolv.conf.tmp"))
            .map_err(|e| format!("Error copying resolv.conf: {}", e)));
        try!(rename(&root.join("etc/resolv.conf.tmp"),
                    &root.join("etc/resolv.conf"),)
            .map_err(|e| format!("Error copying resolv.conf: {}", e)));
    }

    let mut runenv = TreeMap::new();
    for &(ref k, ref v) in os::env().iter() {
        runenv.insert(k.clone(), v.clone());
    }
    env.populate_environ(&mut runenv);


    let pipe = try!(CPipe::new());
    let mut monitor = Monitor::new(true);

    let pid = try!(run_container(&pipe, env, &root, &ropts,
        &env.work_dir, &command, cmdargs.as_slice(), &runenv));

    // TODO(tailhook) set uid map from config
    let uid = unsafe { getuid() };
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

    monitor.add("child".to_string(), pid);
    monitor.wait_all();
    return Ok(monitor.get_status());
}
