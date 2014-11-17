use std::os;
use std::io::stdio::{stdout, stderr};
use std::default::Default;
use std::from_str::FromStr;

use argparse::{ArgumentParser, Store, List, Collect, StoreTrue, StoreFalse};
use collections::treemap::TreeMap;

use super::uidmap::write_uid_map;
use super::monitor::Monitor;
use super::env::Environ;
use super::linux::{ensure_dir, RunOptions, run_container, CPipe};
use super::linux::{Pseudo, Bind, BindRO, BindROTmp};
use super::options::env_options;
use super::userns::IdRanges;
use super::utils::run::write_resolv_conf;

#[deriving(Clone)]
struct Volume {
    source: Path,
    target: Path,
    writeable: bool,
}

impl FromStr for Volume {
    fn from_str(input: &str) -> Option<Volume> {
        let mut split = input.splitn(2, ':');
        let src = match split.next() {
            Some(val) => Path::new(val),
            None => return None,
        };
        let tgt = match split.next() {
            Some(val) => Path::new(val),
            None => return None,
        };
        let flags = match split.next() {
            Some(val) => val,
            None => "ro",
        };
        return Some(Volume {
            source: src,
            target: tgt,
            writeable: match flags {
                "ro" => false,
                "rw" => true,
                _ => return None,
            },
        });
    }
}


pub fn run_chroot(env: &mut Environ, args: Vec<String>)
    -> Result<int, String>
{
    let mut root: Path = Path::new("");
    let mut command: String = "".to_string();
    let mut cmdargs: Vec<String> = Vec::new();
    let mut ropts: RunOptions = Default::default();
    let mut volumes: Vec<Volume> = Vec::new();
    let mut resolv: bool = true;
    let mut uidranges: IdRanges = Vec::new();
    let mut gidranges: IdRanges = Vec::new();
    let mut inventory: bool = false;
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
        ap.refer(&mut inventory)
            .add_option(["--inventory"], box StoreTrue,
                "Mount inventory folder of vagga inside container \
                 /tmp/inventory");
        ap.refer(&mut volumes)
            .metavar("SOURCE:TARGET:FLAGS")
            .add_option(["--volume"], box Collect::<Volume>,
                "Mount folder SOURCE into the directory TARGET inside
                 container. FLAGS is one of 'ro' -- readonly (default),
                 'rw' -- writeable. Note: currently vagga requires existing
                 directory for the mount. This may change in future");
        ap.refer(&mut resolv)
            .add_option(["--no-resolv"], box StoreFalse,
                "Do not copy /etc/resolv.conf");
        ap.refer(&mut uidranges)
            .add_option(["--uid-ranges"], box Store::<IdRanges>,
                "Uid ranges that must be mapped. E.g. 0-1000,65534");
        ap.refer(&mut gidranges)
            .add_option(["--gid-ranges"], box Store::<IdRanges>,
                "Gid ranges that must be mapped. E.g. 0-100,500-1000");
        ap.refer(&mut ropts.uidmap)
            .add_option(["--force-user-namespace"], box StoreTrue,
                "Forces creation usernamespace (by default doesn't create if
                 VAGGA_IN_BUILD is set");
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
        try!(write_resolv_conf(&root, &Path::new("/etc")));
    }

    let mut runenv = TreeMap::new();
    for &(ref k, ref v) in os::env().iter() {
        runenv.insert(k.clone(), v.clone());
    }
    env.populate_environ(&mut runenv);
    let path_root = Path::new("/");
    let mnt_root = env.local_vagga.join(".mnt");
    ropts.mounts.push(Pseudo(
        "tmpfs".to_c_str(), mnt_root.join("tmp").to_c_str(),
        "size=100m,mode=1777".to_c_str()));
    if inventory {
        ropts.mounts.push(BindROTmp(env.vagga_inventory.to_c_str(),
                    mnt_root.join_many(["tmp", "inventory"]).to_c_str()));
    }
    ropts.mounts.extend(volumes.iter().map(|vol| {
        let fullpath = mnt_root.join(
            vol.target.path_relative_from(&path_root).unwrap());
        if vol.writeable {
            Bind(vol.source.to_c_str(), fullpath.to_c_str())
        } else {
            BindRO(vol.source.to_c_str(), fullpath.to_c_str())
        }
    }));


    let pipe = try!(CPipe::new());
    let mut monitor = Monitor::new(true);

    let pid = try!(run_container(&pipe, env, &root, &ropts,
        &env.work_dir, &command, cmdargs.as_slice(), &runenv));

    if ropts.uidmap {
        try!(write_uid_map(pid, &uidranges, &gidranges));
    }

    try!(pipe.wakeup());

    monitor.add("child".to_string(), pid);
    monitor.wait_all();
    return Ok(monitor.get_status());
}
