use std::io::ALL_PERMISSIONS;
use std::rand::{task_rng, Rng};
use std::io::fs::{File, mkdir_recursive};
use std::io::process::{Command, Ignored, InheritFd, ExitStatus};

use super::super::context::{BuildContext, Alpine, Unknown};

static MIRRORS: &'static str = include_str!("../../../alpine/MIRRORS.txt");


#[deriving(Show)]
pub struct AlpineInfo {
    mirror: String,
    version: String,
}


pub fn apk_run(args: &[&str], packages: &[String]) -> Result<(), String> {
    let mut cmd = Command::new("/vagga/bin/apk");
    cmd.stdin(Ignored).stdout(InheritFd(1)).stderr(InheritFd(2))
        .env("PATH", "/vagga/bin")
        .args(args)
        .args(packages);
    debug!("Running APK {}", cmd);
    return match cmd.output()
        .map_err(|e| format!("Can't run apk: {}", e))
        .map(|o| o.status)
    {
        Ok(ExitStatus(0)) => Ok(()),
        Ok(val) => Err(format!("Apk exited with status: {}", val)),
        Err(x) => Err(format!("Error running tar: {}", x)),
    }
}

pub fn setup_base(ctx: &mut BuildContext, version: &String)
    -> Result<(), String>
{
    if let Unknown = ctx.distribution {
    } else {
        return Err(format!("Conflicting distribution"));
    };
    let apk_dir = Path::new("/vagga/root/etc/apk");
    let repos = MIRRORS.split('\n').collect::<Vec<&str>>();
    let mirror = task_rng().choose(repos.as_slice())
        .expect("At least one mirror should work");
    debug!("Chosen mirror {}", mirror);

    try!(mkdir_recursive(&Path::new("/vagga/root/etc/apk/cache"),
                         ALL_PERMISSIONS)
        .map_err(|e| format!("Error creating apk dir: {}", e)));
    try!(ctx.add_cache_dir(Path::new("/etc/apk/cache"),
                           "alpine-cache".to_string()));
    try!(mkdir_recursive(&apk_dir, ALL_PERMISSIONS)
        .map_err(|e| format!("Error creating apk dir: {}", e)));

    try!(File::create(&Path::new("/vagga/root/etc/apk/repositories"))
         .and_then(|mut f|
            writeln!(f, "{}{}/main", mirror, version))
        .map_err(|e| format!("Error creating apk repo: {}", e)));
    try!(apk_run(&[
        "--allow-untrusted",
        "--update-cache",
        "--root", "/vagga/root",
        "--initdb",
        "add",
        "alpine-base",
        ], &[]));
    ctx.distribution = Alpine(AlpineInfo {
        mirror: mirror.to_string(),
        version: version.to_string(),
    });
    Ok(())
}


pub fn install(_ctx: &mut BuildContext, pkgs: &Vec<String>)
    -> Result<(), String>
{
    apk_run(&[
        "--allow-untrusted",
        "--root", "/vagga/root",
        "add",
        ], pkgs.as_slice())
}

pub fn ensure_pip(_ctx: &mut BuildContext, ver: u8) -> Result<Path, String> {
    if ver != 2 {
        return Err(format!("Python {} is not supported", ver));
    }
    try!(apk_run(&[
        "--allow-untrusted",
        "--root", "/vagga/root",
        "add",
        "python",
        "py-pip",
        ], &[]));
    return Ok(Path::new("/usr/bin/pip"));
}
